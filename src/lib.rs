
//use itertools::Itertools;
use pyo3::prelude::*;
use statrs::distribution::{Binomial,Discrete};
use std::cmp::max;
use std::cmp::min;
use std::usize;
use std::vec;
use itertools::iproduct;
use std::collections::HashMap;
use rayon::prelude::*;
use dashmap::DashMap;



mod rust;
// Constants
// D_MAX is the maximum demand for the Poisson or negative binomial distribution.
pub const D_MAX: usize = 25; 

// #[pyfunction]
// fn test_function() -> String {
//     let d1_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf('N', 10.0, Some(0.6));
//     println!("Validation {}", d1_pmf[1]);
//     "Validation complete".to_string()
// }


// #[pyfunction]
// #[pyo3(signature = (state, sa_demand_param_one, sb_demand_param_one, h_w, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None))]
// fn expectation_warehouse(state: (usize,usize,usize), sa_demand_param_one: f64, sb_demand_param_one: f64, h_w: f64, p: Option<f64>, sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>) -> PyResult<f64> {
//     // Generate required parameters and pmf's
//     let dfw_p: f64 = p.unwrap_or(0.8);


//     // Pregenerate the binomial distribution pmf's for all situations
//     // We know the max it can be is the max demand and the min is 0
//     let mut binom_pmf = [[0.0; D_MAX+1]; D_MAX+1];
//     for i in 0..D_MAX+1 {
//         let binom_distr = Binomial::new(dfw_p, i as u64).unwrap();
//         for j in 0..D_MAX+1 {
//             binom_pmf[i][j] = (binom_distr.pmf(j as u64)) as f64;
//         }
//     }
 
//     // Generate demand pmfs
//     let dist: char = distribution.unwrap_or('P');
//     let da_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf(dist, sa_demand_param_one, sa_demand_param_two);
//     let db_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf(dist, sb_demand_param_one, sb_demand_param_two);

//     // Calculate the expectation
//     let mut exp: f64 = 0.0;

//        // First stage shortage
//     for (da_val,da_pmf_i) in da_pmf.iter().enumerate() {
//         let max_beta_sa: usize = min(max(da_val as isize - state.1 as isize,0), state.0 as isize)as usize;
//         for (db_val,db_pmf_i) in db_pmf.iter().enumerate() {
//             for j in 0..max_beta_sa+1 {
//                 let max_beta_sb: usize = min(max(db_val as isize - state.2 as isize,0), (state.0-j) as isize) as usize;
//                 for k in 0..max_beta_sb+1 {
//                     let fs = da_pmf_i * db_pmf_i * binom_pmf[max_beta_sa][j] * binom_pmf[max_beta_sb][k] * h_w * (state.0 - (j+k)) as f64;
//                     exp += fs;
//                 }
//             }
//         }
//     }

//     Ok(exp)
// }


// #[pyfunction]
// #[pyo3(signature = (state, sa_demand_param_one, sb_demand_param_one, h_s, c_u_s, c_p, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None))]
// fn expectation_store(state: (usize,usize,usize), sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, c_u_s: f64, c_p: f64, p: Option<f64>, sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>) -> PyResult<f64> {
//     // Generate required parameters and pmf's
//     let dfw_p: f64 = p.unwrap_or(0.8);
    
//     // Pregenerate the binomial distribution pmf's for all situations
//     // We know the max it can be is the max demand and the min is 0
//     let mut binom_pmf = [[0.0; D_MAX+1]; D_MAX+1];
//     for i in 0..D_MAX+1 {
//         let binom_distr = Binomial::new(dfw_p, i as u64).unwrap();
//         for j in 0..D_MAX+1 {
//             binom_pmf[i][j] = (binom_distr.pmf(j as u64)) as f64;
//         }
//     }
 
//     // Generate demand pmfs
//     let dist: char = distribution.unwrap_or('P');
//     let da_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf(dist, sa_demand_param_one, sa_demand_param_two);
//     let db_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf(dist, sb_demand_param_one, sb_demand_param_two);
    
//     let mut exp = 0.0;
//     // Calculate the expectation
//     // Due to fulfilment of excess demand being indifferent as to the location (since costs and lead-time are identical) we deal with store 1 first then store 2.
//     for (da_val,da_pmf_i) in da_pmf.iter().enumerate() {
//         // Add holding cost of excess demand
//         if (state.1 as isize - da_val as isize) >= 0 {
//             exp += da_pmf_i * h_s * (state.1 as f64 - da_val as f64);
//         } else {
//             let excess = da_val as isize - state.1 as isize;
//             // See how much stock we can fulfil from the warehouse
//             let max_beta_sa: usize = min(max(excess,0), state.0 as isize)as usize;
//             for j in 0..max_beta_sa+1 {
//                 let unfulfilled = excess - j as isize;
//                 exp += da_pmf_i*binom_pmf[max_beta_sa][j]*(c_p*j as f64+unfulfilled as f64*c_u_s) as f64;
//             }
//         }

//     }

//     for (db_val,db_pmf_i) in db_pmf.iter().enumerate() {
//         // Add holding cost of excess demand
//         if (state.2 as isize - db_val as isize) >= 0 {
//             exp += db_pmf_i * h_s * (state.2 as f64 - db_val as f64);
//         } else {
//             // See how much stock we can fulfil from the warehouse
//             // Need to take into accoutn how much we expect to be taken up by store 1
//             for (da_val,da_pmf_i) in da_pmf.iter().enumerate() {
//                 if (state.1 as isize - da_val as isize) >= 0 {
//                     // In this case store 1 took no stock from the warehouse
//                     // Carry on with the procedure
//                     let remaining_warehouse_stock = state.0;
//                     let excess = db_val as isize - state.2 as isize;
//                     // See how much stock we can fulfil from the warehouse
//                     let max_beta_sb: usize = min(max(excess,0), remaining_warehouse_stock as isize)as usize;
//                     for j in 0..max_beta_sb+1 {
//                             let unfulfilled = excess - j as isize;
//                             exp += da_pmf_i*db_pmf_i*binom_pmf[max_beta_sb][j]*(c_p*j as f64+unfulfilled as f64*c_u_s) as f64;
//                     }
                    
//                 } else {
//                     let excess_s1 = da_val as isize - state.1 as isize;
//                     // See how much stock we can fulfil from the warehouse
//                     let max_beta_sa: usize = min(max(excess_s1,0), state.0 as isize)as usize;
//                     for j in 0..max_beta_sa+1 {
//                         // Number of remaining stock we can fulfil from the warehouse
//                         let remaining_warehouse_stock = state.0 - j;
//                         let excess = db_val as isize - state.2 as isize;
//                         // See how much stock we can fulfil from the warehouse
//                         let max_beta_sb: usize = min(max(excess,0), remaining_warehouse_stock as isize)as usize;
//                         for k in 0..max_beta_sb+1 {
//                                 let unfulfilled = excess - k as isize;
//                                 exp += da_pmf_i*db_pmf_i*binom_pmf[max_beta_sb][k]*(c_p*k as f64+unfulfilled as f64*c_u_s) as f64;
//                         }
//                     }
                        
//                 }
//             }
//         }

//     }

//     Ok(exp)
// }


// #[pyfunction]
// #[pyo3(signature = (sa_demand_param_one, sb_demand_param_one, h_s, h_w,c_u_s, c_p, c_ts, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10))]
// fn pre_calculate_warehouse_costs(sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>) -> PyResult<HashMap<(usize,usize,usize),f64>>{
//     let policy_constructor = OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb);
//     let warehouse_dict = policy_constructor.expectation_all_warehouse();
//     Ok(warehouse_dict)
// }

// #[pyfunction]
// #[pyo3(signature = (sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10))]
// fn pre_calculate_store_costs(sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>) -> PyResult<HashMap<(usize,usize,usize),f64>> {
//     let policy_constructor = OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb);
//     let store_dict = policy_constructor.expectation_all_stores();
//     Ok(store_dict)
// }

/// EVERYTHING BELOW THIS LINE IS FOR THE OPTIMAL POLICY FUNCTION
/// 
/// 
struct OptimalPolicy {
    h_s: f64,
    h_w: f64,
    c_u_s: f64,
    c_p: f64,
    c_ts: f64,
    da_pmf: [f64; D_MAX],
    db_pmf: [f64; D_MAX],
    binom_pmf: [[f64; D_MAX+1]; D_MAX+1],
    max_wh: usize,
    max_sa: usize,
    max_sb: usize,
    gamma: f64

}

impl OptimalPolicy {
    fn new(sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>, gamma: Option<f64>) -> Self {
        // Assign optional parameters
        let p: f64 = p.unwrap_or(0.8);
        let distribution: char = distribution.unwrap_or('P');
        let da_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf(distribution, sa_demand_param_one, sa_demand_param_two);
        let db_pmf: [f64; D_MAX] = rust::distributions::generate_distributions::distribution_pmf(distribution, sb_demand_param_one, sb_demand_param_two);
        let mut binom_pmf = [[0.0; D_MAX+1]; D_MAX+1];
        for i in 0..D_MAX+1 {
            let binom_distr = Binomial::new(p, i as u64).unwrap();
            for j in 0..D_MAX+1 {
                binom_pmf[i][j] = (binom_distr.pmf(j as u64)) as f64;
            }
        }

        OptimalPolicy {h_s, h_w, c_u_s, c_p, c_ts, da_pmf, db_pmf, binom_pmf, max_wh: max_wh.unwrap_or(20), max_sa: max_sa.unwrap_or(10), max_sb: max_sb.unwrap_or(10), gamma: gamma.unwrap_or(0.99)}
    }

    // Function to generate the state space
    fn construct_state_space_iterator(&self) -> impl Iterator<Item = (usize, usize, usize)> {
        iproduct!(0..=self.max_wh-1, 0..=self.max_sa-1, 0..=self.max_sb-1)
    }

    fn construct_action_space(&self) -> HashMap<(usize, usize, usize), Vec<(usize, usize, usize, usize, usize)>> {
        let mut state_action_space: HashMap<(usize, usize, usize), Vec<(usize, usize, usize, usize, usize)>> = HashMap::new();
        let state_space_iterator = self.construct_state_space_iterator();
        for state in state_space_iterator {
            let state = (state.0, state.1, state.2);
            let action_space = self.generate_action_space(state);
            state_action_space.insert(state, action_space);
        }
        state_action_space
    }

    fn generate_action_space(&self, state: (usize,usize,usize)) -> Vec<(usize, usize, usize, usize, usize)>  {
        // Generate the action space
        let mut transhipment_options: Vec<(usize, usize)> = vec![(0,0)];

        // Transhipments from store 1 to 2
        for i in 1..min(state.1+1, self.max_sb-state.2) {
            transhipment_options.push((i,0));
        }

        // Transhipments from store 2 to 1
        for i in 1..min(state.2+1, self.max_sa-state.1) {
            transhipment_options.push((0,i));
        }

        let mut action_space = Vec::new();

        for (t_a_to_b, t_b_to_a) in transhipment_options {
            // Update state with TS option
            let new_state = (state.0, state.1 - t_a_to_b + t_b_to_a, state.2 - t_b_to_a + t_a_to_b);
            
            // Go through valid orders
            for order_st_a in 0..max(min(self.max_sa-new_state.1,max(new_state.0 as isize-new_state.1 as isize,0) as usize),1) {
                for order_st_b in 0..max(min(self.max_sb-new_state.2,max(new_state.0 as isize-new_state.2 as isize,0) as usize),1) {
                    // Check if the order is valid
                    if order_st_a + order_st_b <= new_state.0 {
                        for wh_order in 0..self.max_wh-new_state.0 {
                            action_space.push((wh_order,order_st_a, order_st_b, t_a_to_b, t_b_to_a));
                        }   
                    }
                }
            }

        }
        action_space
    }

    fn expectation_warehouse(&self, state: (usize,usize,usize)) -> PyResult<f64> {
        let mut exp: f64 = 0.0;
        // First stage shortage
        for (da_val,da_pmf_i) in self.da_pmf.iter().enumerate() {
            let max_beta_sa: usize = min(max(da_val as isize - state.1 as isize,0), state.0 as isize)as usize;
            for (db_val,db_pmf_i) in self.db_pmf.iter().enumerate() {
                for j in 0..max_beta_sa+1 {
                    let max_beta_sb: usize = min(max(db_val as isize - state.2 as isize,0), (state.0-j) as isize) as usize;
                    for k in 0..max_beta_sb+1 {
                        let fs = da_pmf_i * db_pmf_i * self.binom_pmf[max_beta_sa][j] * self.binom_pmf[max_beta_sb][k] * self.h_w * (state.0 - (j+k)) as f64;
                        exp += fs;
                    }
                }
            }
        }
        Ok(exp)
    }
    
    fn expectation_all_stores(&self) -> HashMap<(usize, usize, usize), f64> {
        let mut state_space = HashMap::new();
        let state_space_iterator = self.construct_state_space_iterator();
        for state in state_space_iterator {
            let state = (state.0, state.1, state.2);
            let exp = self.expectation_store(state).unwrap();
            state_space.insert(state, exp);
        }
        state_space
    }

    fn expectation_all_warehouse(&self) -> HashMap<(usize, usize, usize), f64>  {
        let mut state_space = HashMap::new();
        let state_space_iterator = self.construct_state_space_iterator();
        for state in state_space_iterator {
            let state = (state.0, state.1, state.2);
            let exp = self.expectation_warehouse(state).unwrap();
            state_space.insert(state, exp);
        }
        state_space
    }

    fn expectation_store(&self, state: (usize,usize,usize)) -> PyResult<f64>
    { let mut exp = 0.0;
        // Calculate the expectation
        // Due to fulfilment of excess demand being indifferent as to the location (since costs and lead-time are identical) we deal with store 1 first then store 2.
        for (da_val,da_pmf_i) in self.da_pmf.iter().enumerate() {
            // Add holding cost of excess demand
            if (state.1 as isize - da_val as isize) >= 0 {
                exp += da_pmf_i * self.h_s * (state.1 as f64 - da_val as f64);
            } else {
                let excess = da_val as isize - state.1 as isize;
                // See how much stock we can fulfil from the warehouse
                let max_beta_sa: usize = min(max(excess,0), state.0 as isize)as usize;
                for j in 0..max_beta_sa+1 {
                    let unfulfilled = excess - j as isize;
                    exp += da_pmf_i*self.binom_pmf[max_beta_sa][j]*(self.c_p*j as f64+unfulfilled as f64*self.c_u_s) as f64;
                }
            }
    
        }
    
        for (db_val,db_pmf_i) in self.db_pmf.iter().enumerate() {
            // Add holding cost of excess demand
            if (state.2 as isize - db_val as isize) >= 0 {
                exp += db_pmf_i * self.h_s * (state.2 as f64 - db_val as f64);
            } else {
                // See how much stock we can fulfil from the warehouse
                // Need to take into accoutn how much we expect to be taken up by store 1
                for (da_val,da_pmf_i) in self.da_pmf.iter().enumerate() {
                    if (state.1 as isize - da_val as isize) >= 0 {
                        // In this case store 1 took no stock from the warehouse
                        // Carry on with the procedure
                        let remaining_warehouse_stock = state.0;
                        let excess = db_val as isize - state.2 as isize;
                        // See how much stock we can fulfil from the warehouse
                        let max_beta_sb: usize = min(max(excess,0), remaining_warehouse_stock as isize)as usize;
                        for j in 0..max_beta_sb+1 {
                                let unfulfilled = excess - j as isize;
                                exp += da_pmf_i*db_pmf_i*self.binom_pmf[max_beta_sb][j]*(self.c_p*j as f64+unfulfilled as f64*self.c_u_s) as f64;
                        }
                        
                    } else {
                        let excess_s1 = da_val as isize - state.1 as isize;
                        // See how much stock we can fulfil from the warehouse
                        let max_beta_sa: usize = min(max(excess_s1,0), state.0 as isize)as usize;
                        for j in 0..max_beta_sa+1 {
                            // Number of remaining stock we can fulfil from the warehouse
                            let remaining_warehouse_stock = state.0 - j;
                            let excess = db_val as isize - state.2 as isize;
                            // See how much stock we can fulfil from the warehouse
                            let max_beta_sb: usize = min(max(excess,0), remaining_warehouse_stock as isize)as usize;
                            for k in 0..max_beta_sb+1 {
                                    let unfulfilled = excess - k as isize;
                                    exp += da_pmf_i*db_pmf_i*self.binom_pmf[max_beta_sb][k]*(self.c_p*k as f64+unfulfilled as f64*self.c_u_s) as f64;
                            }
                        }
                            
                    }
                }
            }
    
        }
    
        Ok(exp)
    }


}


fn terminal_cost(policy : &OptimalPolicy, cost: Option<f64>) -> DashMap<(usize, usize, usize), f64> {
    let ss = policy.construct_state_space_iterator();
    let  v_t = DashMap::new();
    let cost = cost.unwrap_or(0.0);
    for state in ss {
        let state = (state.0, state.1, state.2);
        v_t.insert(state, cost);
    }
    v_t
}

// Implementing optimal policy here:



fn future_costs(policy: &OptimalPolicy, state: (usize,usize,usize), orders: (usize,usize,usize), v_t_plus_1: &HashMap<(usize, usize, usize), f64>) -> f64 {
    // Still need to do this
    // Remember as well to add in the cost of the transhipments
    let mut exp = 0.0;
    for (db_val,db_pmf_i) in policy.db_pmf.iter().enumerate() {
        let  sb_post_demand = max(state.2 as isize- db_val as isize,0) as usize + orders.2;
        
        // If the second store can satisfy demand then the warehouse next state is limited just store A
        // easy peasy lemon squeezy
        if (state.2 as isize - db_val as isize) >= 0 {
            for (da_val, da_pmf_i) in policy.da_pmf.iter().enumerate() {
                let  sa_post_demand = max(state.1 as isize - da_val as isize,0) as usize + orders.1;
                if (state.1 as isize - da_val as isize) >= 0 {
                    // No stores took stock therefore the warehouse just adds its order
                    let wh_post_demand = state.0 + orders.0;
                    exp += da_pmf_i*db_pmf_i*v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                } else {
                    // See how much store A asks for 
                    let excess = da_val as isize - state.1 as isize;
                    // See how much stock we can fulfil from the warehouse
                    let max_beta_sa: usize = min(max(excess,0), state.0 as isize) as usize;
                    // Iterate over this and calculate next state probability
                    for j in 0..max_beta_sa+1 {
                        let wh_post_demand = state.0 - j + orders.0;
                        exp += da_pmf_i*db_pmf_i*policy.binom_pmf[max_beta_sa][j]*v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                    }
                }

            }
        } else {
            // In this case store b needs to take stock from the warehouse

            // See if store a took stock from the warehouse
            for (da_val, da_pmf_i) in policy.da_pmf.iter().enumerate() {
                let sa_post_demand = max(state.1 as isize - da_val as isize,0) as usize + orders.1;
                if (state.1 as isize - da_val as isize) >= 0 {
                    // In this case store a took no stock from the warehouse
                    // Carry on with the procedure
                    let remaining_warehouse_stock = state.0;
                    let excess = db_val as isize - state.2 as isize;
                    // See how much stock we can fulfil from the warehouse
                    let max_beta_sb: usize = min(max(excess,0), remaining_warehouse_stock as isize)as usize;
                    let sb_post_demand = max(state.2 as isize - db_val as isize,0) as usize + orders.2;
                            
                    for j in 0..max_beta_sb+1 {
                            let wh_post_demand = state.0 - j + orders.0;
                            exp += da_pmf_i*db_pmf_i*policy.binom_pmf[max_beta_sb][j]*v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]
                    }
                    
                } else {
                    // Store A took stock from the warehouse
                    let excess_s1 = da_val as isize - state.1 as isize;
                    let sa_post_demand = max(state.1 as isize - da_val as isize,0) as usize + orders.1;
                    // See how much stock we can fulfil from the warehouse
                    let max_beta_sa: usize = min(max(excess_s1,0), state.0 as isize)as usize;
                    let sb_post_demand = max(state.2 as isize - db_val as isize,0) as usize + orders.2;
                        
                    for j in 0..max_beta_sa+1 {
                        // Number of remaining stock we can fulfil from the warehouse
                        let remaining_warehouse_stock = state.0 - j;
                        let excess = db_val as isize - state.2 as isize;
                        // See how much stock we can fulfil from the warehouse
                        let max_beta_sb: usize = min(max(excess,0), remaining_warehouse_stock as isize)as usize;
                        for k in 0..max_beta_sb+1 {
                                let wh_post_demand = state.0 - (j+k) + orders.0;
                                exp += da_pmf_i*db_pmf_i*policy.binom_pmf[max_beta_sa][j]*policy.binom_pmf[max_beta_sb][k]*v_t_plus_1[&(wh_post_demand, sa_post_demand, sb_post_demand)]

                                
            }   
    }         
        }

    }
}}
    exp
}

// Calculate the value function and returns best action 
// Action space is of the form: (wh_order, sa_order, sb_order, transhipments 1->2, transhipments 2->1)
fn value_function(policy : &OptimalPolicy, pre_action_state: (usize,usize,usize), v_t_plus_1: &HashMap<(usize, usize, usize), f64>, action_space: &Vec<(usize,usize,usize,usize,usize)>, store_expectation: &HashMap<(usize, usize, usize), f64>, warehouse_expectation: &HashMap<(usize, usize, usize), f64>,) -> ((usize, usize, usize, usize, usize),f64) {
    // generate action space
    let mut best_action:Option<((usize, usize, usize, usize, usize), f64)> = None;
    
    for (wh_order, st_a_order, st_b_order, t_a_to_b, t_b_to_a) in action_space {
        // Post transhipment and store ordering state. Note because of LT=1, the orders don't arrive till the future cost part
        let post_state = (
            pre_action_state.0-st_a_order-st_b_order, // Wh
            pre_action_state.1-t_a_to_b+t_b_to_a, // Store A
            pre_action_state.2-t_b_to_a+t_a_to_b // Store B
        );
        let im_cost = policy.c_ts*(t_a_to_b+t_b_to_a) as f64 + warehouse_expectation[&post_state] + store_expectation[&post_state];
        let fut_cost: f64 = policy.gamma*future_costs(policy, post_state, (*wh_order, *st_a_order, *st_b_order), v_t_plus_1);
        let total_cost = im_cost + fut_cost;
        if best_action.is_none() || total_cost < best_action.unwrap().1 {
            best_action = Some(((*wh_order, *st_a_order, *st_b_order, *t_a_to_b, *t_b_to_a), total_cost));
        }   
    }
    best_action.unwrap()
}

#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn optimal_policy_par(periods: usize, sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, num_cores: Option<usize>, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>, gamma: Option<f64>) -> PyResult<(HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>, HashMap<(usize, usize, usize), f64>)> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb, gamma);
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let action_space = policy_constructor.construct_action_space();

    // Create the thread pool
    rayon::ThreadPoolBuilder::new().num_threads(num_cores.unwrap_or(4)).build_global().unwrap();

    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> = terminal_cost(&policy_constructor, None);
    let optimal_pol: DashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)> = DashMap::new();
    // Iterate through periods
    for t in (1..periods).rev() {
        println!("Period: {:?}",t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize,usize,usize)> = policy_constructor.construct_state_space_iterator().collect();
        println!("Length of state space: {:?}",state_space.len());
        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
            // Calculate the value function
            let (action, v_t_x) = value_function(&policy_constructor, state, &v_plus_1_hm, &action_space[&state], &store_expectation, &warehouse_expectation);
            // Update the value function

            v.insert(state, v_t_x);
            // Store the optimal policy
            optimal_pol.insert((t,state.0,state.1,state.2),action);
        });
        println!();
    }
    let optimal_pol_hm = optimal_pol.clone().into_iter().collect::<HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>>();
    let v_hm = v.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
    Ok((optimal_pol_hm, v_hm))
}

#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn optimal_policy(periods: usize, sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>, gamma: Option<f64>) -> PyResult<(HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>, HashMap<(usize, usize, usize), f64>)> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb,gamma);
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let action_space = policy_constructor.construct_action_space();


    // Load in terminal cost (assume zero for now)
    let mut v: HashMap<(usize, usize, usize), f64> = terminal_cost(&policy_constructor, None).into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
    let mut optimal_pol: HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)> = HashMap::new();
    // Iterate through periods
    for t in (1..periods).rev() {
        println!("Period: {:?}",t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        v.clear(); // Reset V to repopulate

        // Iterate through all states
        for (index, state) in policy_constructor.construct_state_space_iterator().enumerate() {
            print!("\rTesting state {:?} out of {:?}. Current: {:?}",index,policy_constructor.max_wh*policy_constructor.max_sa*policy_constructor.max_sb,state);
            let state = (state.0, state.1, state.2);
            // Calculate the value function
            let (action, v_t_x) = value_function(&policy_constructor, state, &v_plus_1, &action_space[&state], &store_expectation, &warehouse_expectation);
            // Update the value function
            v.insert(state, v_t_x);
            // Store the optimal policy
            optimal_pol.insert((t,state.0,state.1,state.2),action);
           
        }
        println!();
    }
    Ok((optimal_pol, v))
}



/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(optimal_policy, m)?)?;
    m.add_function(wrap_pyfunction!(optimal_policy_par, m)?)?;
    //m.add_function(wrap_pyfunction!(pre_calculate_store_costs, m)?)?;
    //m.add_function(wrap_pyfunction!(pre_calculate_warehouse_costs, m)?)?;
    //m.add_function(wrap_pyfunction!(expectation_warehouse, m)?)?;
    //m.add_function(wrap_pyfunction!(expectation_store, m)?)?;
    Ok(())
}
