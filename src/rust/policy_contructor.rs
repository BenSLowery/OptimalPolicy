//////////////////
//   Contains initialisation for an inventory policy
//   Used a as a basis for calculating an optimal policy or evaluating a policy
//////////////////

use crate::rust;
//use itertools::Itertools;
use itertools::iproduct;
use pyo3::prelude::*;
use statrs::distribution::{Binomial, Discrete};
use std::cmp::max;
use std::cmp::min;
use std::collections::HashMap;
use std::usize;
use std::vec;

pub struct OptimalPolicy {
    pub h_s: f64,
    pub h_w: f64,
    pub c_u_s: f64,
    pub c_p: f64,
    pub c_ts: f64,
    pub da_pmf: [f64; crate::D_MAX],
    pub db_pmf: [f64; crate::D_MAX],
    pub binom_pmf: [[f64; crate::D_MAX + 1]; crate::D_MAX + 1],
    pub max_wh: usize,
    pub max_sa: usize,
    pub max_sb: usize,
    pub gamma: f64,
    pub base_stock_a: usize,
    pub base_stock_b: usize,
}

impl OptimalPolicy {
    pub fn new(
        sa_demand_param_one: f64,
        sb_demand_param_one: f64,
        h_s: f64,
        h_w: f64,
        c_u_s: f64,
        c_p: f64,
        c_ts: f64,
        base_stock_a: usize,
        base_stock_b: usize,
        p: Option<f64>,
        sa_demand_param_two: Option<f64>,
        sb_demand_param_two: Option<f64>,
        distribution: Option<char>,
        max_wh: Option<usize>,
        max_sa: Option<usize>,
        max_sb: Option<usize>,
        gamma: Option<f64>,
    ) -> Self {
        // Assign optional parameters
        let p: f64 = p.unwrap_or(0.8);
        let distribution: char = distribution.unwrap_or('P');
        let da_pmf: [f64; crate::D_MAX] =
            rust::distributions::generate_distributions::distribution_pmf(
                distribution,
                sa_demand_param_one,
                sa_demand_param_two,
            );
        let db_pmf: [f64; crate::D_MAX] =
            rust::distributions::generate_distributions::distribution_pmf(
                distribution,
                sb_demand_param_one,
                sb_demand_param_two,
            );
        let mut binom_pmf = [[0.0; crate::D_MAX + 1]; crate::D_MAX + 1];
        for i in 0..crate::D_MAX + 1 {
            let binom_distr = Binomial::new(p, i as u64).unwrap();
            for j in 0..crate::D_MAX + 1 {
                binom_pmf[i][j] = (binom_distr.pmf(j as u64)) as f64;
            }
        }

        OptimalPolicy {
            h_s,
            h_w,
            c_u_s,
            c_p,
            c_ts,
            da_pmf,
            db_pmf,
            binom_pmf,
            max_wh: max_wh.unwrap_or(20),
            max_sa: max_sa.unwrap_or(10),
            max_sb: max_sb.unwrap_or(10),
            gamma: gamma.unwrap_or(0.99),
            base_stock_a,
            base_stock_b,
        }
    }

    // Function to generate the state space
    pub fn construct_state_space_iterator(&self) -> impl Iterator<Item = (usize, usize, usize)> {
        iproduct!(
            0..=self.max_wh - 1,
            0..=self.max_sa - 1,
            0..=self.max_sb - 1
        )
    }

    pub fn construct_action_space(
        &self,
    ) -> HashMap<(usize, usize, usize), Vec<(usize, usize, usize, usize, usize)>> {
        let mut state_action_space: HashMap<
            (usize, usize, usize),
            Vec<(usize, usize, usize, usize, usize)>,
        > = HashMap::new();
        let state_space_iterator = self.construct_state_space_iterator();
        for state in state_space_iterator {
            let state = (state.0, state.1, state.2);
            let action_space = self.generate_action_space(state);
            state_action_space.insert(state, action_space);
        }
        state_action_space
    }

    pub fn generate_action_space(
        &self,
        state: (usize, usize, usize),
    ) -> Vec<(usize, usize, usize, usize, usize)> {
        // Generate the action space
        let mut transhipment_options: Vec<(usize, usize)> = vec![(0, 0)];

        // Transhipments from store 1 to 2
        for i in 1..min(state.1 + 1, self.max_sb - state.2) {
            transhipment_options.push((i, 0));
        }

        // Transhipments from store 2 to 1
        for i in 1..min(state.2 + 1, self.max_sa - state.1) {
            transhipment_options.push((0, i));
        }

        let mut action_space = Vec::new();

        for (t_a_to_b, t_b_to_a) in transhipment_options {
            // Update state with TS option
            let new_state = (
                state.0,
                state.1 - t_a_to_b + t_b_to_a,
                state.2 - t_b_to_a + t_a_to_b,
            );

            // Go through valid orders
            for order_st_a in 0..max(new_state.0 + 1, 1) {
                if order_st_a + new_state.1 < self.max_sa {
                    for order_st_b in 0..max(new_state.0 + 1, 1) {
                        if order_st_b + new_state.2 < self.max_sb {
                            // Check if the order is valid
                            if order_st_a + order_st_b <= new_state.0 {
                                for wh_order in 0..self.max_wh - new_state.0 {
                                    action_space.push((
                                        wh_order, order_st_a, order_st_b, t_a_to_b, t_b_to_a,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        action_space
    }

    pub fn expectation_warehouse(&self, state: (usize, usize, usize)) -> PyResult<f64> {
        let mut exp: f64 = 0.0;
        // First stage shortage
        for (da_val, da_pmf_i) in self.da_pmf.iter().enumerate() {
            let max_beta_sa: usize =
                min(max(da_val as isize - state.1 as isize, 0), state.0 as isize) as usize;
            for (db_val, db_pmf_i) in self.db_pmf.iter().enumerate() {
                for j in 0..max_beta_sa + 1 {
                    let max_beta_sb: usize = min(
                        max(db_val as isize - state.2 as isize, 0),
                        (state.0 - j) as isize,
                    ) as usize;
                    for k in 0..max_beta_sb + 1 {
                        let fs = da_pmf_i
                            * db_pmf_i
                            * self.binom_pmf[max_beta_sa][j]
                            * self.binom_pmf[max_beta_sb][k]
                            * self.h_w
                            * (state.0 - (j + k)) as f64;
                        exp += fs;
                    }
                }
            }
        }
        Ok(exp)
    }

    pub fn expectation_all_stores(&self) -> HashMap<(usize, usize, usize), f64> {
        let mut state_space = HashMap::new();
        let state_space_iterator = self.construct_state_space_iterator();
        for state in state_space_iterator {
            let state = (state.0, state.1, state.2);
            let exp = self.expectation_store(state).unwrap();
            state_space.insert(state, exp);
        }
        state_space
    }

    pub fn expectation_all_warehouse(&self) -> HashMap<(usize, usize, usize), f64> {
        let mut state_space = HashMap::new();
        let state_space_iterator = self.construct_state_space_iterator();
        for state in state_space_iterator {
            let state = (state.0, state.1, state.2);
            let exp = self.expectation_warehouse(state).unwrap();
            state_space.insert(state, exp);
        }
        state_space
    }

    // Keeps track of all one step ahead expectations for all states and order-up-to levels
    // returns: ((state, store_1/store_2, order-up-to-level), expectation, first stage expectation)
    pub fn all_one_step_ahead_out(&self) -> (HashMap<(usize, usize, usize), (f64, f64)>,HashMap<(usize, usize, usize), (f64, f64)>) {
        let mut one_step_ahead_state_space = HashMap::new();
        let mut one_step_ahead_state_space_terminal = HashMap::new();
        // Store a
        for st_a in 0..self.max_sa {
            let (exp, exp_first_stage) =
                self.one_step_ahead(st_a, 1, self.base_stock_a as f64, false);

            let (exp_terminal, exp_first_stage_terminal) =
                self.one_step_ahead(st_a, 1, self.base_stock_a as f64, true);
            one_step_ahead_state_space.insert((st_a, 1, self.base_stock_a), (exp, exp_first_stage));
            one_step_ahead_state_space_terminal.insert(
                (st_a, 1, self.base_stock_a),
                (exp_terminal, exp_first_stage_terminal),
            );
        }
        // Store b
        for st_b in 0..self.max_sb {
            let (exp, exp_first_stage) =
                self.one_step_ahead(st_b, 2, self.base_stock_b as f64, false);

            let (exp_terminal, exp_first_stage_terminal) =
                self.one_step_ahead(st_b, 2, self.base_stock_b as f64, true);
            one_step_ahead_state_space.insert((st_b, 2, self.base_stock_b), (exp, exp_first_stage));
            one_step_ahead_state_space_terminal.insert(
                (st_b, 2, self.base_stock_b),
                (exp_terminal, exp_first_stage_terminal),
            );
        }

        (one_step_ahead_state_space, one_step_ahead_state_space_terminal)
    }

    pub fn expectation_store(&self, state: (usize, usize, usize)) -> PyResult<f64> {
        let mut exp = 0.0;
        // Calculate the expectation
        // Due to fulfilment of excess demand being indifferent as to the location (since costs and lead-time are identical) we deal with store 1 first then store 2.
        for (da_val, da_pmf_i) in self.da_pmf.iter().enumerate() {
            // Add holding cost of excess demand
            if (state.1 as isize - da_val as isize) >= 0 {
                exp += da_pmf_i * self.h_s * (state.1 as f64 - da_val as f64);
            } else {
                let excess = da_val as isize - state.1 as isize;
                // See how much stock we can fulfil from the warehouse
                let max_beta_sa: usize = min(max(excess, 0), state.0 as isize) as usize;
                for j in 0..max_beta_sa + 1 {
                    let unfulfilled = excess - j as isize;
                    exp += da_pmf_i
                        * self.binom_pmf[max_beta_sa][j]
                        * (self.c_p * j as f64 + unfulfilled as f64 * self.c_u_s) as f64;
                }
            }
        }

        for (db_val, db_pmf_i) in self.db_pmf.iter().enumerate() {
            // Add holding cost of excess demand
            if (state.2 as isize - db_val as isize) >= 0 {
                exp += db_pmf_i * self.h_s * (state.2 as f64 - db_val as f64);
            } else {
                // See how much stock we can fulfil from the warehouse
                // Need to take into accoutn how much we expect to be taken up by store 1
                for (da_val, da_pmf_i) in self.da_pmf.iter().enumerate() {
                    if (state.1 as isize - da_val as isize) >= 0 {
                        // In this case store 1 took no stock from the warehouse
                        // Carry on with the procedure
                        let remaining_warehouse_stock = state.0;
                        let excess = db_val as isize - state.2 as isize;
                        // See how much stock we can fulfil from the warehouse
                        let max_beta_sb: usize =
                            min(max(excess, 0), remaining_warehouse_stock as isize) as usize;
                        for j in 0..max_beta_sb + 1 {
                            let unfulfilled = excess - j as isize;
                            exp += da_pmf_i
                                * db_pmf_i
                                * self.binom_pmf[max_beta_sb][j]
                                * (self.c_p * j as f64 + unfulfilled as f64 * self.c_u_s) as f64;
                        }
                    } else {
                        let excess_s1 = da_val as isize - state.1 as isize;
                        // See how much stock we can fulfil from the warehouse
                        let max_beta_sa: usize = min(max(excess_s1, 0), state.0 as isize) as usize;
                        for j in 0..max_beta_sa + 1 {
                            // Number of remaining stock we can fulfil from the warehouse
                            let remaining_warehouse_stock = state.0 - j;
                            let excess = db_val as isize - state.2 as isize;
                            // See how much stock we can fulfil from the warehouse
                            let max_beta_sb: usize =
                                min(max(excess, 0), remaining_warehouse_stock as isize) as usize;
                            for k in 0..max_beta_sb + 1 {
                                let unfulfilled = excess - k as isize;
                                exp += da_pmf_i
                                    * db_pmf_i
                                    * self.binom_pmf[max_beta_sb][k]
                                    * (self.c_p * k as f64 + unfulfilled as f64 * self.c_u_s)
                                        as f64;
                            }
                        }
                    }
                }
            }
        }

        Ok(exp)
    }

    // For ESR policy
    // Assumes stationary distribution
    pub fn one_step_ahead(
        &self,
        x: usize,
        store: usize,
        st_out: f64,
        terminal_period: bool,
    ) -> (f64, f64) {
        // Calculate the order quantity
        let q: f64 = f64::max(st_out - x as f64, 0.0);

        // Calculate the expectation
        let mut exp: f64 = 0.0;
        let mut exp_first_stage: f64 = 0.0;

        // get which stores pmf to use
        let d_pmf = if store == 1 {
            &self.da_pmf
        } else {
            &self.db_pmf
        };

        for (d1_val, d1_pmf_i) in d_pmf.iter().enumerate() {
            // First stage shortage
            let shortage_p1: usize = max(d1_val as isize - x as isize, 0) as usize;
            for j in 0..shortage_p1 + 1 {
                let fs = d1_pmf_i
                    * self.binom_pmf[shortage_p1][j]
                    * (f64::max(d1_val as f64 - x as f64, 0.0) - j as f64);
                exp_first_stage += fs;
                exp += fs;
            }
            

            // if terminal we have probabiliy of 0 demand as 1.
            let d2_iter_terminal = &[1.0; 1];

            for (d2_val, d2_pmf_i) in if !terminal_period {
                d_pmf.iter().enumerate()
            } else {
                d2_iter_terminal.iter().enumerate()
            } {
                let shortage_p2: usize = max(
                    d2_val as isize - max(x as isize - d1_val as isize, 0) - q as isize,
                    0,
                ) as usize;
                for j in 0..shortage_p2 + 1 {
                    exp += d1_pmf_i
                        * d2_pmf_i
                        * self.binom_pmf[shortage_p2][j]
                        * (f64::max(
                            d2_val as f64 - f64::max(x as f64 - d1_val as f64, 0.0) - q,
                            0.0,
                        ) - j as f64);
                }
            }
        }
        (exp, exp_first_stage)
    }
}
