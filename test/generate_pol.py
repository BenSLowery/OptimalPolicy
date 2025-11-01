# Test all policies
import optimalpolicy._core as rust_helpers
import scipy.stats as sp
import pickle
sa_demand = 5
sb_demand = 2
c_h = 3
c_u_s = 9
c_p = 0
ts = 1
dfw = 0.8
cores = 3
c_h = 1
all_pols = {}

sa = int(sp.poisson(sa_demand*2).ppf(0.99))
sb = int(sp.poisson(sb_demand*2).ppf(0.99))
pol, val_bs = rust_helpers.policy_evaluation_par_bs(
    periods=5,
    sa_demand_param_one=sa_demand,
    sb_demand_param_one=sb_demand,
    h_s=3,
    h_w=1,
    c_u_s=9,
    c_p=0,
    c_ts=1,
    base_stock_vals=(9,0,0),
    p=dfw,
    num_cores=cores,
    max_wh=18,
    max_sa=14,
    max_sb=10,
    transhipment_policy='L',
    gamma=0.999
)
print(min(val_bs, key=val_bs.get),val_bs[min(val_bs, key=val_bs.get)])