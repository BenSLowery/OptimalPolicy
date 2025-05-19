import optimalpolicy.optimal_policy as op
import optimalpolicy.optimal_policy_rust_hybrid as op_rust
import optimalpolicy.optimal_policy_rust as pure_rust
import pickle


pure_rust.test()
#print('Starting test')
#instance = op_rust.optimal_policy(5)
#print(instance.G_s[(0,4,0)])
#print(instance.G_s[(0,0,4)])
#instance.finite_horizon_dp()
#print(len(instance.G_w))
#print(len(instance.G_s))