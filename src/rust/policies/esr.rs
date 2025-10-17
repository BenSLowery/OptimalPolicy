use crate::rust;
use std::collections::HashMap;

use std::usize;

pub fn calculate_esr(
    policy_contructor: &rust::policy_contructor::OptimalPolicy,
    expecation_all_one_step_ahead: &HashMap<(usize, usize, usize), (f64, f64)>,
    state_a: usize,
    state_b: usize, 
    base_stock_a: usize,
    base_stock_b: usize,
    terminal_period: bool
) -> (usize, usize) {
    // Find source and destination node
    let source = if state_a < 1 {
        2
    } else if state_b < 1 {
        1
    // Opposite edn if we're at the max state value then we cannot be a destination
    } else if state_a == policy_contructor.max_sa-1 {
        1
    } else if state_b == policy_contructor.max_sb-1 {
        2
    } else {
        // if both stores have no stock then we cannot transfer anything
        if state_a == 0 && state_b == 0 {
            return (0, 0);
        }
        // Here it can either be a or b so we calculate the alpha/delta to find source and destination
        let f_vals_store_a =  (expecation_all_one_step_ahead[&((state_a-1), 1, base_stock_a)],expecation_all_one_step_ahead[&((state_a), 1, base_stock_a)],expecation_all_one_step_ahead[&((state_a+1), 1, base_stock_a)]);
        let f_vals_store_b =  (expecation_all_one_step_ahead[&((state_b-1), 2, base_stock_b)],expecation_all_one_step_ahead[&((state_b), 2, base_stock_b)],expecation_all_one_step_ahead[&((state_b+1), 2, base_stock_b)]);
        let alpha_a = f_vals_store_a.0.0-f_vals_store_a.1.0;
        let alpha_b = f_vals_store_b.0.0-f_vals_store_b.1.0;
        // Return the smallest value as the relevant source store
        if alpha_a < alpha_b {1} else {2}
    };

    let destination = if source == 1 {2} else {1};

    let mut source_info = (if source == 1 {state_a} else {state_b}, source, if source == 1 {base_stock_a} else {base_stock_b});
    let mut desintation_info = (if destination == 1 {state_a} else {state_b},destination, if destination == 1 {base_stock_a} else {base_stock_b});

    let mut transhipments_still_occur = true;
    


    while transhipments_still_occur {
        // Check we are not at the state-space boundary
        if desintation_info.1 == 1 && desintation_info.0 == policy_contructor.max_sa-1 {
            transhipments_still_occur = false;
        } else if desintation_info.1 == 2 && desintation_info.0 == policy_contructor.max_sb-1 {
            transhipments_still_occur = false;
        
        } // conversly if we are at the minimum state value we cannot transfer more
        else if source_info.0 == 0 {
                transhipments_still_occur = false;
        } else {
            let alpha = expecation_all_one_step_ahead[&(source_info.0-1, source_info.1, source_info.2)].0-expecation_all_one_step_ahead[&(source_info.0, source_info.1, source_info.2)].0;
            let delta = expecation_all_one_step_ahead[&(desintation_info.0, desintation_info.1, desintation_info.2)].0-expecation_all_one_step_ahead[&(desintation_info.0+1, desintation_info.1, desintation_info.2)].0;
            if delta - alpha > policy_contructor.c_ts/policy_contructor.c_u_s {
                // check secondary condition
                if (expecation_all_one_step_ahead[&(desintation_info.0, desintation_info.1, desintation_info.2)].1-expecation_all_one_step_ahead[&(desintation_info.0+1, desintation_info.1, desintation_info.2)].1) >= (expecation_all_one_step_ahead[&(source_info.0-1, source_info.1, source_info.2)].1-expecation_all_one_step_ahead[&(source_info.0, source_info.1, source_info.2)].1) {

                } else {
                    transhipments_still_occur = false;
                }
            } else {
                transhipments_still_occur = false;
            }
            
            if transhipments_still_occur {
                // Make transfer
                source_info.0 -= 1;
                desintation_info.0 += 1;
            }
        }
    }
    if source_info.1 == 1 {
        return (state_a-source_info.0, 0);
    } else {
        return (0, state_b-source_info.0);
    }
}
