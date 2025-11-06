# Test all policies
import optimalpolicy._core as rust_helpers
import scipy.stats as sp
import pickle

optimal_actions = pickle.load(
    open(
        "/home/loweryb/Project2/SimStudyPaper2/two_store_sim_study/test_opt_pol.pkl",
        "rb",
    )
)

val_bs = rust_helpers.policy_evaluation_par_opt(
    50,
    5,
    2,
    2,
    1,
    9,
    0,
    1,
    optimal_actions,
    num_cores=1,
    p=0.8,
    max_wh=25,
    max_sa=15,
    max_sb=9,
    gamma=0.999,
)
print(min(val_bs, key=val_bs.get), val_bs[min(val_bs, key=val_bs.get)])
