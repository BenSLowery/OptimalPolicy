import optimalpolicy._core as rust_helper
import pickle
import pandas as pd
# Instance,T,Store A demand,Store B demand,distribution,holding store,holding warehouse,shortage,dfw cost,transhipment cost,dfw probability,max warehouse,transhipment policy,gamma
def eval_opt_policy(policy):


    wh_exp, st_exp = rust_helper.warehouse_store_expectations_py(
        policy['Store A demand'],
        policy['Store B demand'],
        policy['holding store'],
        policy['holding warehouse'],
        policy['shortage'],
        policy['dfw cost'],
        policy['transhipment cost'],
        p=policy['dfw probability'],
        max_wh=25,
        max_sa=15,
        max_sb=9,
        gamma=0.999,
    )
    return (wh_exp, st_exp)

if __name__ == '__main__':
    sim_study = pd.read_csv('/home/loweryb/Project2/SimStudyPaper2/two_store_sim_study/sim_study_parameters.csv')
    
    wh_exp, st_exp = eval_opt_policy(sim_study[sim_study['Instance'] == 179].iloc[0])
    print(wh_exp)
    print(st_exp)