//use itertools::Itertools;
use dashmap::DashMap;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::usize;

mod rust;
// Constants
// D_MAX is the maximum demand for the Poisson or negative binomial distribution.
pub const D_MAX: usize = 25;

// Policy evaluation given a regular base-stock policy for the action.
// Need to include a variety of different policies.
// Once this is done you just need to call rust::value_function::value_function_pol_eval and pass the normal constructors + action
#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, base_stock_vals=(14,7,7) ,transhipment_policy='N',num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn policy_evaluation_par_bs(
    periods: usize,
    sa_demand_param_one: f64,
    sb_demand_param_one: f64,
    h_s: f64,
    h_w: f64,
    c_u_s: f64,
    c_p: f64,
    c_ts: f64,
    base_stock_vals: Option<(usize, usize, usize)>,
    transhipment_policy: Option<char>,
    num_cores: Option<usize>,
    p: Option<f64>,
    sa_demand_param_two: Option<f64>,
    sb_demand_param_two: Option<f64>,
    distribution: Option<char>,
    max_wh: Option<usize>,
    max_sa: Option<usize>,
    max_sb: Option<usize>,
    gamma: Option<f64>,
) -> PyResult<(
    HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>,
    HashMap<(usize, usize, usize), f64>,
)> {
    let base_stock_policy = base_stock_vals.unwrap_or((14, 7, 7));
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(
        sa_demand_param_one,
        sb_demand_param_one,
        h_s,
        h_w,
        c_u_s,
        c_p,
        c_ts,
        base_stock_policy.1,
        base_stock_policy.2,
        p,
        sa_demand_param_two,
        sb_demand_param_two,
        distribution,
        max_wh,
        max_sa,
        max_sb,
        gamma,
    );
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();

    // Implement transhipment policy
    // Can be 'N' - No transhipment, 'T' - TIE, 'E' - ESR or 'L' - Lookahead Policy
    let transhipment_policy = transhipment_policy.unwrap_or('N');

    let store_a_expectation_mean = rust::distributions::generate_distributions::distribution_mean(
        distribution.unwrap_or('P'),
        sa_demand_param_one,
        sa_demand_param_two,
    );
    let store_b_expectation_mean = rust::distributions::generate_distributions::distribution_mean(
        distribution.unwrap_or('P'),
        sb_demand_param_one,
        sb_demand_param_two,
    );

    // generate the one step ahead expectations to use later (pregenerated hashmap for easier reading)
    let one_step_ahead_expectations = if transhipment_policy == 'E' {
        policy_constructor.all_one_step_ahead_out()
    } else {
        (HashMap::new(), HashMap::new()) // Create empty hashmap if not needed
    };

    // generate one step lookahead expectations for the lookahead if needed
    let one_step_lookahead_expectations = if transhipment_policy == 'L' {
        policy_constructor.all_one_step_ahead_la(store_a_expectation_mean, store_b_expectation_mean)
    } else {
        (HashMap::new(), HashMap::new()) // Create empty hashmap if not needed
    };

    // Create the thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cores.unwrap_or(4))
        .build()
        .unwrap();

    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> =
        rust::value_function::terminal_cost(&policy_constructor, None);
    let optimal_pol: DashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)> =
        DashMap::new();
    // Iterate through periods

    for t in (1..periods).rev() {
        println!("Period: {:?}", t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize, usize, usize)> = policy_constructor
            .construct_state_space_iterator()
            .collect();
        //println!("Length of state space: {:?}",state_space.len());
        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1
                .clone()
                .into_iter()
                .collect::<HashMap<(usize, usize, usize), f64>>();

            let ordering_action: (usize, usize, usize);
            let transhipment_action: (usize, usize);

            if transhipment_policy == 'L' {
                let final_period = t == periods - 1;
                let lookahead_action = rust::policies::lookahead::calculate_lookahead(
                    &policy_constructor,
                    &one_step_lookahead_expectations,
                    state,
                    base_stock_policy.0,
                    final_period,
                );
                ordering_action = (lookahead_action.0, lookahead_action.1, lookahead_action.2);
                transhipment_action = (lookahead_action.3, lookahead_action.4);
            } else {
                transhipment_action = if transhipment_policy == 'N' {
                    (0 as usize, 0 as usize)
                } else if transhipment_policy == 'T' {
                    rust::policies::tie::calculate_tie(
                        state.1,
                        state.2,
                        store_a_expectation_mean,
                        store_b_expectation_mean,
                        max_sa.unwrap_or(10),
                        max_sb.unwrap_or(10),
                    )
                } else if transhipment_policy == 'E' {
                    let final_period = t == periods - 1;
                    rust::policies::esr::calculate_esr(
                        &policy_constructor,
                        &one_step_ahead_expectations,
                        state.1,
                        state.2,
                        base_stock_policy.1,
                        base_stock_policy.2,
                        final_period,
                    )
                } else {
                    panic!("Transhipment policy not recognised");
                };
                // println!("State: {:?}, Transhipment action: {:?}", state, transhipment_action);
                ordering_action = rust::policies::base_stock::regular_base_stock(
                    (
                        state.0,
                        state.1 - transhipment_action.0 + transhipment_action.1,
                        state.2 - transhipment_action.1 + transhipment_action.0,
                    ),
                    base_stock_policy.0,
                    (base_stock_policy.1, base_stock_policy.2),
                );
            }

            let action = (
                ordering_action.0,
                ordering_action.1,
                ordering_action.2,
                transhipment_action.0,
                transhipment_action.1,
            );

            // Calculate the value function
            let v_t_x = rust::value_function::value_function_pol_eval(
                &policy_constructor,
                state,
                &v_plus_1_hm,
                action,
                &store_expectation,
                &warehouse_expectation,
            );
            // Update the value function

            v.insert(state, v_t_x);
            // Store the optimal policy
            optimal_pol.insert((t, state.0, state.1, state.2), action);
        });
        //println!();
    }
    let optimal_pol_hm = optimal_pol
        .clone()
        .into_iter()
        .collect::<HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>>();
    let v_hm = v
        .clone()
        .into_iter()
        .collect::<HashMap<(usize, usize, usize), f64>>();
    Ok((optimal_pol_hm, v_hm))
}

// Policy evaluation of the optimal action
#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, optimal_actions,num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn policy_evaluation_par_opt(
    periods: usize,
    sa_demand_param_one: f64,
    sb_demand_param_one: f64,
    h_s: f64,
    h_w: f64,
    c_u_s: f64,
    c_p: f64,
    c_ts: f64,
    optimal_actions: HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>,
    num_cores: Option<usize>,
    p: Option<f64>,
    sa_demand_param_two: Option<f64>,
    sb_demand_param_two: Option<f64>,
    distribution: Option<char>,
    max_wh: Option<usize>,
    max_sa: Option<usize>,
    max_sb: Option<usize>,
    gamma: Option<f64>,
) -> PyResult<HashMap<(usize, usize, usize), f64>> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(
        sa_demand_param_one,
        sb_demand_param_one,
        h_s,
        h_w,
        c_u_s,
        c_p,
        c_ts,
        0, // Optinal doesn't need base-stock in the policy construcutor
        0, // Optinal doesn't need base-stock in the policy construcutor
        p,
        sa_demand_param_two,
        sb_demand_param_two,
        distribution,
        max_wh,
        max_sa,
        max_sb,
        gamma,
    );
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();

    // Create the thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cores.unwrap_or(4))
        .build()
        .unwrap();

    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> =
        rust::value_function::terminal_cost(&policy_constructor, None);
    // Iterate through periods

    for t in (1..periods).rev() {
        //println!("Period: {:?}",t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize, usize, usize)> = policy_constructor
            .construct_state_space_iterator()
            .collect();
        //println!("Length of state space: {:?}",state_space.len());
        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1
                .clone()
                .into_iter()
                .collect::<HashMap<(usize, usize, usize), f64>>();

            let action = optimal_actions
                .get(&(t, state.0, state.1, state.2))
                .unwrap_or(&(0, 0, 0, 0, 0));
            //println!("{:?}, {:?}", state,action);
            // Calculate the value function
            let v_t_x = rust::value_function::value_function_pol_eval(
                &policy_constructor,
                state,
                &v_plus_1_hm,
                *action,
                &store_expectation,
                &warehouse_expectation,
            );
            // Update the value function

            v.insert(state, v_t_x);
        });
        //println!();
    }
    let v_hm = v
        .clone()
        .into_iter()
        .collect::<HashMap<(usize, usize, usize), f64>>();
    Ok(v_hm)
}

// Optimal Policy
#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn optimal_policy_par(
    periods: usize,
    sa_demand_param_one: f64,
    sb_demand_param_one: f64,
    h_s: f64,
    h_w: f64,
    c_u_s: f64,
    c_p: f64,
    c_ts: f64,
    num_cores: Option<usize>,
    p: Option<f64>,
    sa_demand_param_two: Option<f64>,
    sb_demand_param_two: Option<f64>,
    distribution: Option<char>,
    max_wh: Option<usize>,
    max_sa: Option<usize>,
    max_sb: Option<usize>,
    gamma: Option<f64>,
) -> PyResult<(
    HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>,
    HashMap<(usize, usize, usize), f64>,
)> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(
        sa_demand_param_one,
        sb_demand_param_one,
        h_s,
        h_w,
        c_u_s,
        c_p,
        c_ts,
        0, // Optinal doesn't need base-stock in the policy construcutor
        0, // Optinal doesn't need base-stock in the policy construcutor
        p,
        sa_demand_param_two,
        sb_demand_param_two,
        distribution,
        max_wh,
        max_sa,
        max_sb,
        gamma,
    );
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let action_space = policy_constructor.construct_action_space();

    //println!("{:?}",action_space);

    // Create the thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cores.unwrap_or(4))
        .build()
        .unwrap();

    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> =
        rust::value_function::terminal_cost(&policy_constructor, None);
    let optimal_pol: DashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)> =
        DashMap::new();
    // Iterate through periods
    for t in (1..periods).rev() {
        println!("Period: {:?}", t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize, usize, usize)> = policy_constructor
            .construct_state_space_iterator()
            .collect();
        //println!("Length of state space: {:?}",state_space.len());

        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1
                .clone()
                .into_iter()
                .collect::<HashMap<(usize, usize, usize), f64>>();
            // Calculate the value function
            let (action, v_t_x) = rust::value_function::value_function_optimal_pol(
                &policy_constructor,
                state,
                &v_plus_1_hm,
                &action_space[&state],
                &store_expectation,
                &warehouse_expectation,
            );
            // Update the value function

            v.insert(state, v_t_x);
            // Store the optimal policy
            optimal_pol.insert((t, state.0, state.1, state.2), action);
        });
        println!();
    }
    let optimal_pol_hm = optimal_pol
        .clone()
        .into_iter()
        .collect::<HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>>();
    let v_hm = v
        .clone()
        .into_iter()
        .collect::<HashMap<(usize, usize, usize), f64>>();
    Ok((optimal_pol_hm, v_hm))
}

#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn optimal_policy(
    periods: usize,
    sa_demand_param_one: f64,
    sb_demand_param_one: f64,
    h_s: f64,
    h_w: f64,
    c_u_s: f64,
    c_p: f64,
    c_ts: f64,
    p: Option<f64>,
    sa_demand_param_two: Option<f64>,
    sb_demand_param_two: Option<f64>,
    distribution: Option<char>,
    max_wh: Option<usize>,
    max_sa: Option<usize>,
    max_sb: Option<usize>,
    gamma: Option<f64>,
) -> PyResult<(
    HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>,
    HashMap<(usize, usize, usize), f64>,
)> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(
        sa_demand_param_one,
        sb_demand_param_one,
        h_s,
        h_w,
        c_u_s,
        c_p,
        c_ts,
        0, // Optinal doesn't need base-stock in the policy construcutor
        0, // Optinal doesn't need base-stock in the policy construcutor
        p,
        sa_demand_param_two,
        sb_demand_param_two,
        distribution,
        max_wh,
        max_sa,
        max_sb,
        gamma,
    );
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let action_space = policy_constructor.construct_action_space();

    // Load in terminal cost (assume zero for now)
    let mut v: HashMap<(usize, usize, usize), f64> =
        rust::value_function::terminal_cost(&policy_constructor, None)
            .into_iter()
            .collect::<HashMap<(usize, usize, usize), f64>>();
    let mut optimal_pol: HashMap<
        (usize, usize, usize, usize),
        (usize, usize, usize, usize, usize),
    > = HashMap::new();
    // Iterate through periods
    for t in (1..periods).rev() {
        println!("Period: {:?}", t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        v.clear(); // Reset V to repopulate

        // Iterate through all states
        for (index, state) in policy_constructor
            .construct_state_space_iterator()
            .enumerate()
        {
            print!(
                "\rTesting state {:?} out of {:?}. Current: {:?}",
                index,
                policy_constructor.max_wh * policy_constructor.max_sa * policy_constructor.max_sb,
                state
            );
            let state = (state.0, state.1, state.2);
            // Calculate the value function
            let (action, v_t_x) = rust::value_function::value_function_optimal_pol(
                &policy_constructor,
                state,
                &v_plus_1,
                &action_space[&state],
                &store_expectation,
                &warehouse_expectation,
            );
            // Update the value function
            v.insert(state, v_t_x);
            // Store the optimal policy
            optimal_pol.insert((t, state.0, state.1, state.2), action);
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
    m.add_function(wrap_pyfunction!(policy_evaluation_par_bs, m)?)?;
    m.add_function(wrap_pyfunction!(policy_evaluation_par_opt, m)?)?;
    //m.add_function(wrap_pyfunction!(pre_calculate_store_costs, m)?)?;
    //m.add_function(wrap_pyfunction!(pre_calculate_warehouse_costs, m)?)?;
    //m.add_function(wrap_pyfunction!(expectation_warehouse, m)?)?;
    //m.add_function(wrap_pyfunction!(expectation_store, m)?)?;
    Ok(())
}
