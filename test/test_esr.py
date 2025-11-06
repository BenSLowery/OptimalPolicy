import optimalpolicy._core as rust_helpers
import scipy.stats as sp

sa_demand = 5
sb_demand = 2
c_h = 1
c_u_s = 9
c_p = 0
ts = 1
dfw = 0.8
cores = 3
c_h = 1
for wh in [i for i in range(7, 12)]:
    for alpha in [0.7, 0.75, 0.8, 0.85, 0.9, 0.95]:
        print("Calculating for wh={}, alpha={}".format(wh, alpha))
        sa = int(sp.poisson(sa_demand * 2).ppf(alpha))
        sb = int(sp.poisson(sb_demand * 2).ppf(alpha))
        pol, val_bs = rust_helpers.policy_evaluation_par_bs(
            periods=50,
            sa_demand_param_one=sa_demand,
            sb_demand_param_one=sb_demand,
            h_s=c_h,
            h_w=1,
            c_u_s=9,
            c_p=0,
            c_ts=ts,
            base_stock_vals=(wh, sa, sb),
            p=dfw,
            num_cores=cores,
            max_wh=18,
            max_sa=5 + sa * 2,
            max_sb=5 + sb * 2,
            transhipment_policy="E",
            gamma=0.999,
        )
