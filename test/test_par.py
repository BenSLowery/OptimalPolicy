import optimalpolicy._core as rust_helpers
import pickle

def test():
    pol, val = rust_helpers.optimal_policy_par(5,5,2,1,1,9,0,1,max_wh=32, max_sa=16,max_sb=16)
    # (periods,sa_demand_param_one, sb_demand_param_one, h_s,h_w, c_u_s, c_p, c_ts, num_cores=4, p=None, sa_demand_param_two=None, sb_demand_param_two=None, distribution=None, max_wh=20, max_sa=10, max_sb=10))
    pickle.dump(pol, open("policy_par.pkl", "wb"))
    pickle.dump(val, open("VF_par.pkl", "wb"))  


if __name__ == "__main__":
    test()
    # pol, val = rust_helpers.optimal_policy(5,2,2,1,1,9,0,1)
    # pickle.dump(pol, open("policy.pkl", "wb"))
    # pickle.dump(val, open("VF.pkl", "wb"))
