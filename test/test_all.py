# Test all policies
import optimalpolicy._core as rust_helpers
import scipy.stats as sp
import pickle
sa_demand = 5
sb_demand = 2
c_h = 1
c_u_s = 9
c_p = 0
ts = 1
dfw = 0.8
cores = 3
c_h = 1
all_pols = {}
for wh in [i for i in range(7,14)]: 
    for policy in ['T', 'E', 'N', 'L']:
        if policy != 'L':
            for alpha in [0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 0.99]:
                print('Calculating for wh={}, alpha={}'.format(wh, alpha))
                sa = int(sp.poisson(sa_demand*2).ppf(alpha))
                sb = int(sp.poisson(sb_demand*2).ppf(alpha))
                pol, val_bs = rust_helpers.policy_evaluation_par_bs(
                    periods=5,
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
                    max_wh=18,
                    max_sa=5+sa*2,
                    max_sb=5+sb*2,
                    transhipment_policy=policy,
                    gamma=0.999
                )
                all_pols['({},{}, {})'.format(wh,alpha,policy)] = (min(val_bs, key=val_bs.get),val_bs[min(val_bs, key=val_bs.get)])   
        else:
            alpha = 0
            print('Calculating for wh={}'.format(wh))
            pol, val_bs = rust_helpers.policy_evaluation_par_bs(
                periods=5,
                sa_demand_param_one=sa_demand,
                sb_demand_param_one=sb_demand,
                h_s=c_h,
                h_w=1,
                c_u_s=9,
                c_p=0,
                c_ts=ts,
                base_stock_vals=(wh,0,0),
                p=dfw,
                num_cores=cores,
                max_wh=18,
                max_sa=5+sa*2,
                max_sb=5+sb*2,
                transhipment_policy=policy,
                gamma=0.999
            )
            all_pols['({},{}, {})'.format(wh,alpha,policy)] = (min(val_bs, key=val_bs.get),val_bs[min(val_bs, key=val_bs.get)])
pickle.dump(all_pols,open('test/validate/all_policy_results.pkl','wb'))