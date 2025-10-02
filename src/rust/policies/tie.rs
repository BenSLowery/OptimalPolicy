use rand::prelude::*;
use std::cmp::max;

pub fn calculate_tie(
    state_store_a: usize,
    state_store_b: usize,
    demand_store_a: f64,
    demand_store_b: f64,
    max_sa : usize,
    max_sb : usize,
) -> (usize, usize) {

    let max_sa = (max_sa-1) as f64;
    let max_sb = (max_sb-1) as f64;
    let mut rebalanced_store_a = f64::min((demand_store_a as f64 / (demand_store_a + demand_store_b))
        * (state_store_a + state_store_b) as f64,max_sa as f64) as f64;
    let mut rebalanced_store_b = f64::min((demand_store_b as f64 / (demand_store_a + demand_store_b))
        * (state_store_a + state_store_b) as f64, max_sb as f64) as f64;

    // Check if integer
    if rebalanced_store_a.fract() == 0.0 && rebalanced_store_b.fract() == 0.0 {
        let store_b_a_transhipment =
            max(rebalanced_store_a as isize - state_store_a as isize, 0) as usize;
        let store_a_b_transhipment =
            max(rebalanced_store_b as isize - state_store_b as isize, 0) as usize;
        return (store_a_b_transhipment, store_b_a_transhipment as usize);
    } else {
        // Just round rather than randomly allocate here (for two stores its equivalent)
        let rebalanced_store_a_min = rebalanced_store_a.floor();
        let rebalanced_store_b_min = rebalanced_store_b.floor();
        let excess = ((rebalanced_store_a - rebalanced_store_a_min)
            + (rebalanced_store_b - rebalanced_store_b_min)) as usize;

        if excess > 1 {
            panic!("Excess greater than 1");
        }
        // Randomly allocate excess
        let mut rng = rand::rng();
         // If we're already at the limit of the state space then just add to the other store
        let add_to_a = if rebalanced_store_a_min == max_sa as f64 {
            false
        } else if rebalanced_store_b_min == max_sb as f64 {
            true
        } else {
            rng.random_bool(0.5)
        }; // If to add the excess demand to store a

       

        rebalanced_store_a = if add_to_a {
            rebalanced_store_a_min + 1.0
        } else {
            rebalanced_store_a_min
        };
        rebalanced_store_b =  if add_to_a {
            rebalanced_store_b_min
        } else {
            rebalanced_store_b_min + 1.0
        };
        let store_b_a_transhipment =
            max(rebalanced_store_a as isize - state_store_a as isize, 0) as usize;
        let store_a_b_transhipment =
            max(rebalanced_store_b as isize - state_store_b as isize, 0) as usize;

        return (store_a_b_transhipment, store_b_a_transhipment);
    }
}
