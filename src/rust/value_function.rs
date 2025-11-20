// Calculate a value function based on a given input state
use crate::rust;
//use itertools::Itertools;
use dashmap::DashMap;
use std::cmp::max;
use std::cmp::min;
use std::collections::HashMap;
use std::usize;

// Calculate the value function and returns best action
// Action space is of the form: (wh_order, sa_order, sb_order, transhipments 1->2, transhipments 2->1)
pub fn value_function_optimal_pol(
    policy: &rust::policy_contructor::OptimalPolicy,
    pre_action_state: (usize, usize, usize),
    v_t_plus_1: &HashMap<(usize, usize, usize), f64>,
    action_space: &Vec<(usize, usize, usize, usize, usize)>,
    store_expectation: &HashMap<(usize, usize, usize), f64>,
    warehouse_expectation: &HashMap<(usize, usize, usize), f64>,
) -> ((usize, usize, usize, usize, usize), f64) {
    // generate action space
    let mut best_action: Option<((usize, usize, usize, usize, usize), f64)> = None;

    for (wh_order, st_a_order, st_b_order, t_a_to_b, t_b_to_a) in action_space {
        // Post transhipment and store ordering state. Note because of LT=1, the orders don't arrive till the future cost part
        let post_state = (
            pre_action_state.0 - st_a_order - st_b_order, // Wh
            pre_action_state.1 - t_a_to_b + t_b_to_a,     // Store A
            pre_action_state.2 - t_b_to_a + t_a_to_b,     // Store B
        );
        let im_cost = policy.c_ts * (t_a_to_b + t_b_to_a) as f64
            + warehouse_expectation[&post_state]
            + store_expectation[&post_state];
        let fut_cost: f64 = policy.gamma
            * future_costs(
                policy,
                post_state,
                (*wh_order, *st_a_order, *st_b_order),
                v_t_plus_1,
            );
        let total_cost = im_cost + fut_cost;
        if best_action.is_none() || total_cost < best_action.unwrap().1 {
            best_action = Some((
                (*wh_order, *st_a_order, *st_b_order, *t_a_to_b, *t_b_to_a),
                total_cost,
            ));
        }
    }
    best_action.unwrap()
}

// Calculate the value function given an action has been submitted
pub fn value_function_pol_eval(
    policy: &rust::policy_contructor::OptimalPolicy,
    pre_action_state: (usize, usize, usize),
    v_t_plus_1: &HashMap<(usize, usize, usize), f64>,
    action: (usize, usize, usize, usize, usize),
    store_expectation: &HashMap<(usize, usize, usize), f64>,
    warehouse_expectation: &HashMap<(usize, usize, usize), f64>,
) -> f64 {
    // generate action space
    let (wh_order, st_a_order, st_b_order, t_a_to_b, t_b_to_a) = action;
    // Post transhipment and store ordering state. Note because of LT=1, the orders don't arrive till the future cost part
    let post_state = (
        pre_action_state.0 - st_a_order - st_b_order, // Wh
        pre_action_state.1 - t_a_to_b + t_b_to_a,     // Store A
        pre_action_state.2 - t_b_to_a + t_a_to_b,     // Store B
    );
    let im_cost = policy.c_ts * (t_a_to_b + t_b_to_a) as f64
        + warehouse_expectation[&post_state]
        + store_expectation[&post_state];
    let fut_cost: f64 = policy.gamma
        * future_costs(
            policy,
            post_state,
            (wh_order, st_a_order, st_b_order),
            v_t_plus_1,
        );
    let total_cost = im_cost + fut_cost;
    total_cost
}

pub fn future_costs(
    policy: &rust::policy_contructor::OptimalPolicy,
    state: (usize, usize, usize),
    orders: (usize, usize, usize),
    v_t_plus_1: &HashMap<(usize, usize, usize), f64>,
) -> f64 {
    let mut exp = 0.0;
    for (db_val, db_pmf_i) in policy.db_pmf.iter().enumerate() {
        let sb_post_demand = max(state.2 as isize - db_val as isize, 0) as usize + orders.2;

        // If the second store can satisfy demand then the warehouse next state is limited just store A
        // easy peasy lemon squeezy
        if (state.2 as isize - db_val as isize) >= 0 {
            for (da_val, da_pmf_i) in policy.da_pmf.iter().enumerate() {
                let sa_post_demand = max(state.1 as isize - da_val as isize, 0) as usize + orders.1;
                if (state.1 as isize - da_val as isize) >= 0 {
                    // No stores took stock therefore the warehouse just adds its order
                    let wh_post_demand = state.0 + orders.0;
                    exp += da_pmf_i
                        * db_pmf_i
                        * v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                } else {
                    // See how much store A asks for
                    let excess = da_val as isize - state.1 as isize;
                    // See how much stock we can fulfil from the warehouse
                    let max_beta_sa: usize = min(max(excess, 0), state.0 as isize) as usize;
                    // Iterate over this and calculate next state probability
                    for j in 0..max_beta_sa + 1 {
                        let wh_post_demand = state.0 - j + orders.0;
                        exp += da_pmf_i
                            * db_pmf_i
                            * policy.binom_pmf[max_beta_sa][j]
                            * v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                    }
                }
            }
        } else {
            // In this case store b needs to take stock from the warehouse

            // See if store a took stock from the warehouse
            for (da_val, da_pmf_i) in policy.da_pmf.iter().enumerate() {
                let sa_post_demand = max(state.1 as isize - da_val as isize, 0) as usize + orders.1;
                if (state.1 as isize - da_val as isize) >= 0 {
                    // In this case store a took no stock from the warehouse
                    let excess = db_val as isize - state.2 as isize;
                    // See how much stock we can fulfil from the warehouse
                    let max_beta_sb: usize = min(max(excess, 0), state.0 as isize) as usize;
                    //let sb_post_demand = max(state.2 as isize - db_val as isize,0) as usize + orders.2;

                    for j in 0..max_beta_sb + 1 {
                        let wh_post_demand = state.0 - j + orders.0;
                        exp += da_pmf_i
                            * db_pmf_i
                            * policy.binom_pmf[max_beta_sb][j]
                            * v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                    }
                } else {
                    // Store A took stock from the warehouse
                    let excess_s1 = da_val as isize - state.1 as isize;
                    let sa_post_demand =
                        max(state.1 as isize - da_val as isize, 0) as usize + orders.1;
                    // See how much stock we can fulfil from the warehouse
                    let max_beta_sa: usize = min(max(excess_s1, 0), state.0 as isize) as usize;
                    //let sb_post_demand = max(state.2 as isize - db_val as isize,0) as usize + orders.2;

                    for j in 0..max_beta_sa + 1 {
                        // Number of remaining stock we can fulfil from the warehouse
                        let remaining_warehouse_stock = state.0 - j;
                        let excess = db_val as isize - state.2 as isize;
                        // See how much stock we can fulfil from the warehouse
                        let max_beta_sb: usize =
                            min(max(excess, 0), remaining_warehouse_stock as isize) as usize;
                        for k in 0..max_beta_sb + 1 {
                            let wh_post_demand = state.0 - (j + k) + orders.0;
                            exp += da_pmf_i
                                * db_pmf_i
                                * policy.binom_pmf[max_beta_sa][j]
                                * policy.binom_pmf[max_beta_sb][k]
                                * v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                        }
                    }
                }
            }
        }
    }
    exp
}

pub fn terminal_cost(
    policy: &rust::policy_contructor::OptimalPolicy,
    cost: Option<f64>,
) -> DashMap<(usize, usize, usize), f64> {
    let ss = policy.construct_state_space_iterator();
    let v_t = DashMap::new();
    let cost = cost.unwrap_or(0.0);
    for state in ss {
        let state = (state.0, state.1, state.2);
        v_t.insert(state, cost);
    }
    v_t
}
