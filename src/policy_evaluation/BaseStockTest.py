import optimalpolicy._core as rust_helpers
import pickle
import sys
def test(wh, sa, sb):
    pol, val = rust_helpers.policy_evaluation_par_opt(11,5,2,1,1,9,9,1,base_stock_vals=(wh,sa,sb), p=0.5,num_cores=3,max_wh=28, max_sa=18,max_sb=15, gamma=0.999)
    return val

    

