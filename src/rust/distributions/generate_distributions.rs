use statrs::distribution::{Poisson, NegativeBinomial,Discrete};

pub fn distribution_pmf(dist_type: char, param_1: f64, param_2: Option<f64>) -> [f64; crate::D_MAX] {

    if dist_type == 'P' {
        // Poisson

        // Error check if the demand is too large
        if param_1 > 5.0 {
            panic!("Demand param has to be less than 28, currently {}. To fix increase D_MAX", param_1);
        }
        let mut param_1 = param_1;
        if param_1 == 0.0 {
            param_1 = 0.0001;
        }

        let poisson_distr = Poisson::new(param_1).unwrap();
        let pmf: [f64; crate::D_MAX] = core::array::from_fn(|i| poisson_distr.pmf(i as u64));
        return pmf;

    } else if dist_type == 'N' {
        // Error check if the demand is too large
        // We want the mean + variance is less than 9
        let nbinom_mean: f64  = (param_1*(1.0-param_2.unwrap()))/param_2.unwrap();
        let nbinom_var: f64 = (param_1*(1.0-param_2.unwrap()))/(param_2.unwrap()*param_2.unwrap());
        if nbinom_mean + nbinom_var > 9.0 {
            panic!("Params mean + variance have to be les than 9, currently {}. To fix increase D_MAX and then update the error checking in generate_distributions.rs", nbinom_mean + nbinom_var);
        }

        // Negative Binomial    
        let param_2 = param_2.expect("You need to provide a second parameter for the negative binomial distribution");
        let neg_binom_distr = NegativeBinomial::new(param_1, param_2).unwrap();
        let pmf: [f64; crate::D_MAX] = core::array::from_fn(|i| neg_binom_distr.pmf(i as u64));
        return pmf;
    } else {
        panic!("Distribution type not recognised");
    }
}