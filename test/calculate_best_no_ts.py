import optimalpolicy._core as rust_helpers
import pickle
import sys
import scipy.stats as sp

sa_demand = 5
sb_demand = 2
c_h = 1
c_u_s = 9
c_p = 0
ts = 1
dfw = 0.2
cores = 3

pol_eval_res = {}
for wh in range(7,15): 
    for alpha in [0.5,0.55, 0.6, 0.65, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99]:
        sa = int(sp.poisson(sa_demand*2).ppf(alpha))
        sb = int(sp.poisson(sb_demand*2).ppf(alpha))
        val_bs = rust_helpers.policy_evaluation_par_bs(
            periods=11,
            sa_demand_param_one=sa_demand,
            sb_demand_param_one=sb_demand,
            h_s=c_h,
            h_w=1,
            c_u_s=9,
            c_p=0,
            c_ts=ts,
            base_stock_vals=(wh,sa,sb),
            p=dfw,
            num_cores=cores,
            max_wh=25,
            max_sa=10+sa_demand*2,
            max_sb=10+sb_demand*2,
            gamma=0.999
        )
        pol_eval_res[(wh,alpha)] = (min(val_bs, key=val_bs.get),val_bs[min(val_bs, key=val_bs.get)])
print(pol_eval_res)