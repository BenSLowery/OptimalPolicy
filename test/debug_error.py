import optimalpolicy._core as rust_helpers
import pickle
import sys
def test_optimal_pol():
    #h_s: f64, h_w: f64, c_u_s: f64, c_p: f64, c_ts: f64, 
    pol, val = rust_helpers.optimal_policy_par(
        periods=4,
        sa_demand_param_one=1,
        sb_demand_param_one=1,
        h_s=1,
        h_w=1,
        c_u_s=9,
        c_p=0,
        c_ts=1,
        p=0.5,
        num_cores=3,
        max_wh=12,
        max_sa=5,
        max_sb=5, 
        gamma=0.999
    )
    return val


if __name__ == '__main__':
    val = test_optimal_pol()
    print(val[min(val,key=val.get)])