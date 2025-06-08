import optimalpolicy.optimal_policy as op

import policy_evaluation.BaseStockTest as pure_rust
import pickle
import sys
import scipy.stats as sp

if __name__ == "__main__":
    sa = sp.poisson(5).ppf(0.99)
    sb = sp.poisson(2).ppf(0.99)

    pol = pure_rust.test(int(8), int(sa), int(sb))
    print("({},{}): {},".format(8, 0.99, pol))
    with open('output.out','ab') as f:
        f.write("({},{}): {}\n".format(sys.argv[1], sys.argv[2], pol).encode('utf-8'))