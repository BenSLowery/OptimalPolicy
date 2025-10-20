use crate::rust;
use std::collections::HashMap;
use std::cmp::max;
use std::usize;

pub fn calculate_lookahead (
    policy_constructor: &rust::policy_contructor::OptimalPolicy,
    expectation_all_one_step_lookahead_and_terminal: &(
        HashMap<(usize, usize, usize), (f64, f64, f64)>,
        HashMap<(usize, usize, usize), (f64, f64, f64)>,
    ),
    state: (usize, usize,usize),
    warehouse_order: usize,
    terminal_period: bool,
) -> (usize, usize, usize, usize, usize) {
    let state_a = state.1;
    let state_b = state.2;
    let wh = state.0;
    
    // Save action to return
    let mut action: (usize, usize, usize, usize,usize) = (0,0,0,0,0);

    // remember warehouse calculation
    let expecation_all_one_step_lookahead = if terminal_period {
        &expectation_all_one_step_lookahead_and_terminal.1
    } else {
        &expectation_all_one_step_lookahead_and_terminal.0
    };

    // Calculate transhipment
    (action.1, action.2, action.3, action.4) = if state_a == 0 && state_b == 0 {
        let q_a = expecation_all_one_step_lookahead[&(wh, state_a, 1)].2;
        let q_b = expecation_all_one_step_lookahead[&(wh, state_b, 2)].2;
        (q_a as usize, q_b as usize, 0, 0)
    } else {
        // Find source and destination node
        let source = if state_a < 1 {
            2
        } else if state_b < 1 {
            1
        // Opposite edn if we're at the max state value then we cannot be a destination
        } else if state_a == policy_constructor.max_sa - 1 {
            1
        } else if state_b == policy_constructor.max_sb - 1 {
            2
        } else {
            // Here it can either be a or b so we calculate the alpha/delta to find source and destination
            let f_vals_store_a = (
                expecation_all_one_step_lookahead[&(wh, (state_a - 1), 1)],
                expecation_all_one_step_lookahead[&(wh, (state_a), 1)],
                expecation_all_one_step_lookahead[&(wh, (state_a + 1), 1)],
            );
            let f_vals_store_b = (
                expecation_all_one_step_lookahead[&(wh, (state_b - 1), 2)],
                expecation_all_one_step_lookahead[&(wh, (state_b), 2)],
                expecation_all_one_step_lookahead[&(wh, (state_b + 1), 2)],
            );
            let alpha_a = f_vals_store_a.0 .0 - f_vals_store_a.1 .0;
            let alpha_b = f_vals_store_b.0 .0 - f_vals_store_b.1 .0;
            // Return the smallest value as the relevant source store
            if alpha_a < alpha_b {
                1
            } else {
                2
            }
        };
        let destination = if source == 1 { 2 } else { 1 };

        let mut source_info = (
            wh,
            if source == 1 { state_a } else { state_b },
            source,
        );
        let mut desintation_info = (
            wh,
            if destination == 1 { state_a } else { state_b },
            destination,

        );

        let mut transhipments_still_occur = true;

        while transhipments_still_occur {
            // Check we are not at the state-space boundary
            if desintation_info.2 == 1 && desintation_info.1 == policy_constructor.max_sa - 1 {
                transhipments_still_occur = false;
            } else if desintation_info.2 == 2 && desintation_info.1 == policy_constructor.max_sb - 1 {
                transhipments_still_occur = false;
            }
            // conversly if we are at the minimum state value we cannot transfer more
            else if source_info.1 == 1 {
                transhipments_still_occur = false;
            } else {
                let alpha = expecation_all_one_step_lookahead
                    [&(source_info.0, source_info.1 - 1, source_info.2)]
                    .0
                    - expecation_all_one_step_lookahead[&(source_info.0, source_info.1, source_info.2)].0;
                let delta = expecation_all_one_step_lookahead
                    [&(desintation_info.0, desintation_info.1, desintation_info.2)]
                    .0
                    - expecation_all_one_step_lookahead[&(
                        desintation_info.0,
                        desintation_info.1-1,
                        desintation_info.2,
                    )]
                        .0;
                if delta - alpha > policy_constructor.c_ts / policy_constructor.c_u_s {
                    // check secondary condition
                    if (expecation_all_one_step_lookahead
                        [&(desintation_info.0, desintation_info.1, desintation_info.2)]
                        .1
                        - expecation_all_one_step_lookahead[&(
                            desintation_info.0,
                            desintation_info.1 + 1,
                            desintation_info.2,
                        )]
                            .1)
                        >= (expecation_all_one_step_lookahead
                            [&(source_info.0, source_info.1 -1 , source_info.2)]
                            .1
                            - expecation_all_one_step_lookahead
                                [&(source_info.0, source_info.1, source_info.2)]
                                .1)
                    {
                    } else {
                        transhipments_still_occur = false;
                    }
                } else {
                    transhipments_still_occur = false;
                }

                if transhipments_still_occur {
                    // Make transfer
                    source_info.1 -= 1;
                    desintation_info.1 += 1;
                }
            }
        }
        // Get ordering quantity based on the this state
        let q_source = expecation_all_one_step_lookahead[&(source_info.0, source_info.1, source_info.2)].2;
        let q_destination = expecation_all_one_step_lookahead[&(desintation_info.0, desintation_info.1, desintation_info.2)].2;
        if source_info.1 == 1 {

            (q_source as usize, q_destination as usize, state_a - source_info.1, 0)
        } else {
            (q_destination as usize, q_source as usize, 0, state_b - source_info.1)
        }
    };
    // Calculate warehouse order (uses regular base-stock policy)
    action.0 = max(warehouse_order as usize - max(wh - (action.1 + action.2),0),0);
    action
}
