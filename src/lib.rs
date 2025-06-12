
//use itertools::Itertools;
use pyo3::prelude::*;
use std::usize;
use std::collections::HashMap;
use rayon::prelude::*;
use dashmap::DashMap;




mod rust;
// Constants
// D_MAX is the maximum demand for the Poisson or negative binomial distribution.
pub const D_MAX: usize = 25; 


// Policy evaluation given a regular base-stock policy for the action.
// TODO: add in transhipment base-stock policies.
// Need to include a variety of different policies.
// Once this is done you just need to call rust::value_function::value_function_pol_eval and pass the normal constructors + action
#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, base_stock_vals=(14,7,7) ,num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn policy_evaluation_par_bs(periods: usize, sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64,  base_stock_vals:Option<(usize, usize, usize)>, num_cores: Option<usize>, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>, gamma: Option<f64>) -> PyResult<(HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>, HashMap<(usize, usize, usize), f64>)> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb, gamma);
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let base_stock_policy = base_stock_vals.unwrap_or((14, 7, 7));
    // Create the thread pool
    rayon::ThreadPoolBuilder::new().num_threads(num_cores.unwrap_or(4)).build().unwrap();
    
    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> = rust::value_function::terminal_cost(&policy_constructor, None);
    let optimal_pol: DashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)> = DashMap::new();
    // Iterate through periods

    for t in (1..periods).rev() {
        //println!("Period: {:?}",t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize,usize,usize)> = policy_constructor.construct_state_space_iterator().collect();
        //println!("Length of state space: {:?}",state_space.len());
        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();

            let ordering_action: (usize, usize, usize) = rust::policies::base_stock::regular_base_stock(state, base_stock_policy.0, (base_stock_policy.1,base_stock_policy.2));
            // TODO: transhipment
            let action = (ordering_action.0,ordering_action.1,ordering_action.2,0 as usize,0 as usize);
            //println!("{:?}, {:?}", state,action);
            // Calculate the value function
            let v_t_x = rust::value_function::value_function_pol_eval(&policy_constructor, state, &v_plus_1_hm, action, &store_expectation, &warehouse_expectation);
            // Update the value function

            v.insert(state, v_t_x);
            // Store the optimal policy
            optimal_pol.insert((t,state.0,state.1,state.2),action);
        });
        //println!();
    }
    let optimal_pol_hm = optimal_pol.clone().into_iter().collect::<HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>>();
    let v_hm = v.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
    Ok((optimal_pol_hm, v_hm))
}



// Policy evaluation of the optimal action
#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, optimal_actions,num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn policy_evaluation_par_opt(periods: usize, sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64,  optimal_actions:HashMap<(usize, usize, usize,usize), (usize, usize, usize, usize, usize)>, num_cores: Option<usize>, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>, gamma: Option<f64>) -> PyResult<HashMap<(usize, usize, usize), f64>> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb, gamma);
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    // Create the thread pool
    rayon::ThreadPoolBuilder::new().num_threads(num_cores.unwrap_or(4)).build().unwrap();
    
    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> = rust::value_function::terminal_cost(&policy_constructor, None);
    // Iterate through periods

    for t in (1..periods).rev() {
        //println!("Period: {:?}",t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize,usize,usize)> = policy_constructor.construct_state_space_iterator().collect();
        //println!("Length of state space: {:?}",state_space.len());
        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();

            // TODO: transhipment
            let action = optimal_actions.get(&(t,state.0,state.1,state.2)).unwrap_or(&(0,0,0,0,0));
            //println!("{:?}, {:?}", state,action);
            // Calculate the value function
            let v_t_x = rust::value_function::value_function_pol_eval(&policy_constructor, state, &v_plus_1_hm, *action, &store_expectation, &warehouse_expectation);
            // Update the value function

            v.insert(state, v_t_x);
        });
        //println!();
    }
    let v_hm = v.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
    Ok(v_hm)
}



// Optimal Policy
#[pyfunction]
#[pyo3(signature = (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10, gamma=0.99))]
fn optimal_policy_par(periods: usize, sa_demand_param_one: f64, sb_demand_param_one: f64, h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, num_cores: Option<usize>, p: Option<f64>,sa_demand_param_two: Option<f64>, sb_demand_param_two: Option<f64>, distribution: Option<char>, max_wh: Option<usize>, max_sa: Option<usize>, max_sb: Option<usize>, gamma: Option<f64>) -> PyResult<(HashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)>, HashMap<(usize, usize, usize), f64>)> {
    // Stores all the infrastructure for the parameters in the optimal policy
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb, gamma);
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let action_space = policy_constructor.construct_action_space();

    //println!("{:?}",action_space);

    // Create the thread pool
    rayon::ThreadPoolBuilder::new().num_threads(num_cores.unwrap_or(4)).build().unwrap();

    // Load in terminal cost (assume zero for now)
    let v: DashMap<(usize, usize, usize), f64> = rust::value_function::terminal_cost(&policy_constructor, None);
    let optimal_pol: DashMap<(usize, usize, usize, usize), (usize, usize, usize, usize, usize)> = DashMap::new();
    // Iterate through periods
    for t in (1..periods).rev() {
        println!("Period: {:?}",t);
        // Save previous iteration (v_t+1)
        let v_plus_1 = v.clone();
        //v.clear(); // Reset V to repopulate

        // Iterate through all states
        let state_space: Vec<(usize,usize,usize)> = policy_constructor.construct_state_space_iterator().collect();
        //println!("Length of state space: {:?}",state_space.len());
        
        state_space.clone().par_iter().for_each(|state| {
            //println!("Testing state {:?} out of {:?}. Current: {:?}",state, state_space.len(), state);
            let state = (state.0, state.1, state.2);
            let v_plus_1_hm = v_plus_1.clone().into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
            // Calculate the value function
            let (action, v_t_x) = rust::value_function::value_function_optimal_pol(&policy_constructor, state, &v_plus_1_hm, &action_space[&state], &store_expectation, &warehouse_expectation);
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
    let policy_constructor = rust::policy_contructor::OptimalPolicy::new(sa_demand_param_one, sb_demand_param_one, h_s, h_w, c_u_s, c_p, c_ts, p, sa_demand_param_two, sb_demand_param_two, distribution, max_wh, max_sa, max_sb,gamma);
    let store_expectation = policy_constructor.expectation_all_stores();
    let warehouse_expectation = policy_constructor.expectation_all_warehouse();
    let action_space = policy_constructor.construct_action_space();


    // Load in terminal cost (assume zero for now)
    let mut v: HashMap<(usize, usize, usize), f64> = rust::value_function::terminal_cost(&policy_constructor, None).into_iter().collect::<HashMap<(usize, usize, usize), f64>>();
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
            let (action, v_t_x) = rust::value_function::value_function_optimal_pol(&policy_constructor, state, &v_plus_1, &action_space[&state], &store_expectation, &warehouse_expectation);
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
    m.add_function(wrap_pyfunction!(policy_evaluation_par_bs, m)?)?;
    m.add_function(wrap_pyfunction!(policy_evaluation_par_opt, m)?)?;
    //m.add_function(wrap_pyfunction!(pre_calculate_store_costs, m)?)?;
    //m.add_function(wrap_pyfunction!(pre_calculate_warehouse_costs, m)?)?;
    //m.add_function(wrap_pyfunction!(expectation_warehouse, m)?)?;
    //m.add_function(wrap_pyfunction!(expectation_store, m)?)?;
    Ok(())
}
