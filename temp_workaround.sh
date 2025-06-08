#!/bin/bash
for i in {7..20}; do 
    for j in 0.5 0.55 0.6 0.65 0.7 0.75 0.8 0.85 0.9 0.95 0.99; do
        echo "Running with wh=$i, CF=$j"
        uv run test/validate_pol_eval.py $i $j 
    done
done 