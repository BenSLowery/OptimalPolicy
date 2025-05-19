import optimalpolicy.optimal_policy as op
import optimalpolicy.optimal_policy_rust_hybrid as op_rust
import pickle

print('Starting test')
instance = op_rust.optimal_policy(5)
#print(instance.G_s[(0,4,0)])
#print(instance.G_s[(0,0,4)])
#instance.finite_horizon_dp()
pickle.dump([instance.G_w, instance.G_s],open('./test/validate/instance_rust_python_test.pkl','wb'))

# With two stores
# Maybe try negative binomial distribution with r=2 and p=0.5? - this gives us a mean of 2 and variance of 4
# Then another thats poisson with lambda=2

# State space: 2000 for Poisson, 6750 for Negative Binomial (assuming max demand cut off is 10 for poisson and 15 for negbin, and warehouse cutouff is (num stores)*(cut off))
# Action Space: 