import optimalpolicy._core as rust_helpers
import pickle


def test():
    pol, val = rust_helpers.optimal_policy_par(5, 2, 2, 1, 1, 9, 0, 1)
    pickle.dump(pol, open("policy.pkl", "wb"))
    pickle.dump(val, open("VF.pkl", "wb"))
