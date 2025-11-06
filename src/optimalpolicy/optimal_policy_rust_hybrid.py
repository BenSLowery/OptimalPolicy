import optimalpolicy._core as rust_helpers


# Runs the optimal policy for 2 stores (and a warehouse) for the transhipemnt problem under partial lost-sales
# Assumptions:
# Warehouse Lead time can be up to L^{(w)}
# Store lead time is 1 (inventory arrives at the beginning of the next period)
# Transhipments arrive instantly

import itertools
import scipy.stats as sp


class optimal_policy:
    def __init__(
        self,
        periods,
        wh_lead_time=1,
        partial_lost_sales_chance=0.8,
        partial_lost_sales_cost=0,
        transhipment_cost=1,
        holding_warehouse=1,
        holding_store=1,
        lost_sales_cost=18,
        discount_factor=0.999,
        store_demand_distribution=[["Poisson", [2]], ["Poisson", [2]]],
        warehouse_demand_truncation=20,
        store_demand_truncation=[10, 10],
        num_cores=3,
    ):
        self.T = periods
        self.l_w = wh_lead_time
        self.p = partial_lost_sales_chance
        self.c_p = partial_lost_sales_cost
        self.c_ts = transhipment_cost
        self.h_w = holding_warehouse
        self.h_s = holding_store
        self.c_u = lost_sales_cost  # a.k.a. underage cost
        self.gamma = discount_factor
        self.s1_demand = store_demand_distribution[0]
        self.s2_demand = store_demand_distribution[1]
        self.max_wh = warehouse_demand_truncation
        self.max_s1 = store_demand_truncation[0]
        self.max_s2 = store_demand_truncation[1]

        # Action space is limited by the maximum demand we can experience
        self.max_wh_action = warehouse_demand_truncation
        self.max_s1_action = store_demand_truncation[0]
        self.max_s2_action = store_demand_truncation[1]

        self.num_cores = num_cores  # Number of cores to use for parallel processing

        # Currently only support Warehouse LT=1 so make this clear
        if self.l_w != 1:
            raise ValueError("Currently only support Warehouse Lead Time > 1")

        # Key information for the DP
        self.optimal_pol = {p: {} for p in range(1, self.T + 1)}
        self.V = {}
        self.G_w = {}
        self.G_s = {}

        # Run some pre-processing steps (in a separate function so less messy)
        self._pre_processing()

    def _pre_processing(self):
        """
        Pre-process the demand distributions
        Generate the state space
        Populate the immediate cost function G
        """
        print("Pre-processing steps...")

        # The state space is of the form (w,w+1,...,w+L-1, s1, s2)
        self.state_space = [
            x
            for x in itertools.product(
                [i for i in range(self.max_wh) for j in range(self.l_w)],
                [i for i in range(self.max_s1)],
                [i for i in range(self.max_s2)],
            )
        ]
        print("State space size:", len(self.state_space))
        # Generate demand distributions pmfs
        self.s1_d_pmf = self._generate_pmf(self.s1_demand, self.max_s1)
        self.s2_d_pmf = self._generate_pmf(self.s2_demand, self.max_s2)

        # Pre-calculate part of the immediate cost function G (i.e. everything but the transhipment cost)
        # This just is used for more efficiancy in the DP algorithm
        for idx, state in enumerate(self.state_space):
            print(idx, end="\r")
            self.G_w[state] = rust_helpers.expectation_warehouse(
                tuple(state),
                self.s1_demand[1][0],
                self.s2_demand[1][0],
                self.h_w,
                self.p,
            )
            self.G_s[state] = rust_helpers.expectation_store(
                tuple(state),
                self.s1_demand[1][0],
                self.s2_demand[1][0],
                self.h_w,
                self.c_u,
                self.c_p,
                self.p,
            )

        print("Done.")

    def _generate_pmf(self, demand_dist, max_d):
        # Check which distribution we are checking
        if demand_dist[0] == "Poisson":
            distr = sp.poisson(demand_dist[1][0])

        elif demand_dist[0] == "NegBin":
            # Generate new random negative binomial
            distr = sp.nbinom(demand_dist[1][0], demand_dist[1][1])

        # Generate value/pmf pairings
        d_pmf = [(v, distr.pmf(v)) for v in range(max_d)]
        return d_pmf

    def immediate_cost_warehouse(self, state):
        # Store the remaining warehouse inventory
        exp = 0
        for d_1, d_1_pmf in self.s1_d_pmf:
            max_beta_s1 = min(max(d_1 - state[1], 0), state[0])
            beta_1_pmf = [
                (v, sp.binom.pmf(v, max_beta_s1, self.p))
                for v in range(max_beta_s1 + 1)
            ]
            for d_2, d_2_pmf in self.s2_d_pmf:
                for b1, b1_pmf in beta_1_pmf:
                    max_beta_s2 = min(max(d_2 - state[2], 0), state[0] - b1)
                    beta_2_pmf = [
                        (v, sp.binom.pmf(v, max_beta_s2, self.p))
                        for v in range(max_beta_s2 + 1)
                    ]
                    for b2, b2_pmf in beta_2_pmf:
                        exp += (
                            d_1_pmf
                            * d_2_pmf
                            * b1_pmf
                            * b2_pmf
                            * self.h_w
                            * (state[0] - (b1 + b2))
                        )

        return exp

    def immediate_cost_store(self, state, store_1_demand, store_2_demand):
        """
        Need to factor that the beta for the second store is dependent on the demand of the first store
        """
        wh = state[0]
        s1 = state[1]
        s2 = state[2]

        exp = 0

        # Store 1 first
        for d_1, d_1_pmf in store_1_demand:
            # Penalty for lost sales or holding cost for leftover inventory
            exp += d_1_pmf * (self.h_s * max(s1 - d_1, 0) + self.c_u * max(d_1 - s1, 0))

            # Depends on demand and how much inventory at the warehouse we have to satisfy excess
            max_beta = min(max(d_1 - s1, 0), wh)

            # If we have excess demand, we need to consider how many units we fulfil via partial lost sales
            if max_beta > 0:
                # Find the beta, beta_pmf combination (we could precalculate this?)
                beta_pmf = [
                    (v, sp.binom.pmf(v, max_beta, self.p)) for v in range(max_beta)
                ]
                for b, b_pmf in beta_pmf:
                    # Add the cost of partial lost sales
                    # (here we have the cost of choosing partial lost sales minus any
                    # potential lost sales we incurred intially before the partial lost sales)
                    exp += d_1_pmf * b_pmf * (self.c_p * b - self.c_u * b)

        # Store 2 next, note that the wh is now limited by the demand that is used in store 1 (tbd.)
        for d_2, d_2_pmf in store_2_demand:
            # Penalty for lost sales or holding cost for leftover inventory
            exp += d_2_pmf * (self.h_s * max(s2 - d_2, 0) + self.c_u * max(d_2 - s2, 0))

            # Depends on demand and how much inventory at the warehouse we have to satisfy excess
            max_beta = min(max(d_2 - s2, 0), wh)

            # If we have excess demand, we need to consider how many units we fulfil via partial lost sales
            if max_beta > 0:
                # Find the beta, beta_pmf combination (we could precalculate this?)
                beta_pmf = [
                    (v, sp.binom.pmf(v, max_beta, self.p)) for v in range(max_beta + 1)
                ]
                for b, b_pmf in beta_pmf:
                    # Add the cost of partial lost sales
                    # (here we have the cost of choosing partial lost sales minus any
                    # potential lost sales we incurred intially before the partial lost sales)
                    exp += d_2_pmf * b_pmf * (self.c_p * b - self.c_u * b)

        return exp

    def future_cost(
        self, post_action_state, wh_order, st_1_order, st_2_order, V_t_plus_1
    ):
        exp = 0
        wh = post_action_state[0]
        st_1 = post_action_state[1]
        st_2 = post_action_state[2]

        for d_1, d_1_pmf in self.s1_d_pmf:
            st_1_next_state = max(st_1 - d_1, 0) + st_1_order
            for d_2, d_2_pmf in self.s2_d_pmf:
                st_2_next_state = max(st_2 - d_2, 0) + st_2_order
                inner_exp = 0
                max_beta_s1 = min(max(d_1 - st_1, 0), wh)
                max_beta_s2 = min(max(d_2 - st_2, 0), wh)

                beta_1_pmf = [
                    (v, sp.binom.pmf(v, max_beta_s1, self.p))
                    for v in range(max_beta_s1 + 1)
                ]
                for b1, b1_pmf in beta_1_pmf:
                    max_beta_s2 = min(max(d_2 - st_2, 0), wh - b1)
                    beta_2_pmf = [
                        (v, sp.binom.pmf(v, max_beta_s2, self.p))
                        for v in range(max_beta_s2 + 1)
                    ]
                    for b2, b2_pmf in beta_2_pmf:
                        wh_next_state = wh + wh_order - (b1 + b2)
                        inner_exp += (
                            b1_pmf
                            * b2_pmf
                            * V_t_plus_1[
                                (wh_next_state, st_1_next_state, st_2_next_state)
                            ]
                        )
                exp += d_1_pmf * d_2_pmf * inner_exp

        return exp

    def terminal_cost(self, cost=0):
        """
        Calculate the terminal cost for a given state
        Improve to allow any cost structure
        """
        for state in self.state_space:
            self.V[state] = sum([s * cost for s in state])

    def generate_action_space(self, wh, st_1_pre_ts, st_2_pre_ts):
        # Calculate valid transhipments that can be made
        # Maybe we can precalculate the action set for each state?

        # the Structure of an action is as follows
        # (wh_order, st_1_order, st_2_order, st1->st2 transhipment, st2->st1 transhipment)
        # We cant add more to the other store than it can hold (i.e. the truncation)
        ts_options = (
            [(0, 0)]
            + [
                (i, 0)
                for i in range(1, min(st_1_pre_ts + 1, self.max_s2 - st_2_pre_ts))
            ]
            + [
                (0, i)
                for i in range(1, min(st_2_pre_ts + 1, self.max_s1 - st_1_pre_ts))
            ]
        )
        valid_actions = []
        for ts_s1, ts_s2 in ts_options:
            # Update the state
            st_1 = st_1_pre_ts - ts_s1 + ts_s2
            st_2 = st_2_pre_ts - ts_s2 + ts_s1
            # Go through all valid orders
            for order_st_1 in range(max(min(self.max_s1_action - st_1, wh - st_1), 1)):
                for order_st_2 in range(
                    max(min(self.max_s2_action - st_2, wh - st_2), 1)
                ):
                    if order_st_1 + order_st_2 <= wh:
                        for wh_order in range(self.max_wh_action - wh):
                            valid_actions.append(
                                (wh_order, order_st_1, order_st_2, ts_s1, ts_s2)
                            )
        return valid_actions

    def value_function(self, state, V_t_plus_1):
        """
        Calculate the value function for a given state
        """
        wh = state[0]
        st_1 = state[1]
        st_2 = state[2]

        total_cost_all_actions = {}

        # Calculate valid orders
        actions = self.generate_action_space(wh, st_1, st_2)
        for wh_order, st_1_order, st_2_order, st_1_ts, st_2_ts in actions:
            post_ts_order_state = (
                wh - st_1_order - st_2_order,
                st_1 - st_1_ts + st_2_ts,
                st_2 - st_2_ts + st_1_ts,
            )

            # Immediate cost (warehouse_cost + transhipment cost + store cost)
            im_cost = (
                self.c_ts * (st_1_ts + st_2_ts)
                + self.G_w[post_ts_order_state]
                + self.G_s[post_ts_order_state]
            )

            # Future cost
            fut_cost = self.gamma * self.future_cost(
                post_ts_order_state, wh_order, st_1_order, st_2_order, V_t_plus_1
            )
            total_cost_all_actions[
                (wh_order, st_1_order, st_2_order, st_1_ts, st_2_ts)
            ] = im_cost + fut_cost

        # Find the best action
        best_action = min(total_cost_all_actions, key=total_cost_all_actions.get)
        return [best_action, total_cost_all_actions[best_action]]

    def finite_horizon_dp(self):
        """
        Run DP algorithm to find the optimal policy
        """

        # Step 0. Initialize the value function at the terminal time point
        self.terminal_cost()

        # Step 1. Iterate backwards through all periods
        for period in range(self.T, 0, -1):
            V_t_plus_1 = self.V.copy()  # Get a copt of the value function for the next period . As we will repopulate V in each period
            self.V = {}
            print("Period:", period)
            # Step 2. Enumerate through all poissible states
            for state in self.state_space:
                # Step 3. Caclulate value function and Return best action given the state
                state_action_cost_pair = self.value_function(state, V_t_plus_1)
                # Step 4. Update the value function
                self.V[state] = state_action_cost_pair[1]
                # Step 5. Store the optimal policy for all states in this time period
                self.optimal_pol[period][state] = state_action_cost_pair[0]
        print("Done")
