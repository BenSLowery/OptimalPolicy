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

for c_h in [3]:
    for dfw in [0.8]:
        print('Calculating for c_h={}, dfw={}'.format(c_h, dfw))
        pol_eval_res = {}
        for wh in [13]: 
            for alpha in [0.4]:
                print('Calculating for wh={}, alpha={}'.format(wh, alpha))
                sa = int(sp.poisson(sa_demand*2).ppf(alpha))
                sb = int(sp.poisson(sb_demand*2).ppf(alpha))
                pol, val_bs = rust_helpers.policy_evaluation_par_bs(
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
                    max_wh=14,
                    max_sa=20,
                    max_sb=15,
                    transhipment_policy='T',
                    gamma=0.999
                )
                pol_eval_res['({},{})'.format(wh,alpha)] = (min(val_bs, key=val_bs.get),val_bs[min(val_bs, key=val_bs.get)])
        pickle.dump(pol,open('res.pkl','wb'))