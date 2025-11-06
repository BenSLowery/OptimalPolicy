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
wh = 8
alpha = 0.9
print("Calculating for wh={}, alpha={}".format(wh, alpha))
sa = int(sp.poisson(sa_demand * 2).ppf(alpha))
sb = int(sp.poisson(sb_demand * 2).ppf(alpha))
pol, val_bs = rust_helpers.policy_evaluation_par_bs(
    periods=10,
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
    transhipment_policy="L",
    gamma=0.999,
)
