Rust optimal policy implementation for a 2 store transhipment network with a warehouse, partial lost-sales and transhipments.

Ordering Policies:
* 'R': Regular base-stock
* 'C': Capped base-stock

Transhipment Policies:
* 'E': Expected Shortage Reduction
* 'T': Transhipment Inventory Equalisation
* 'N': No transhipment

Integrated Policies:
* 'O': one step lookahead with no transhipments
* 'L': one step lookahead

Combinations as arguments to `policy_evaluation_par_bs` function:
* TIE: `transhipment_policy='T', ordering_policy='R'` and set `base_stock_vals=(WH, SA, SB)`
* CTIE: `transhipment_policy='T', ordering_policy='C'` and set `base_stock_vals=(WH, SA, SB), order_cap=(SA,SB)`
* NTS: `transhipment_policy='N', ordering_policy='R'` and set `base_stock_vals=(WH, SA, SB)`
* CNTS: `transhipment_policy='N', ordering_policy='C'` and set `base_stock_vals=(WH, SA, SB), order_cap=(SA,SB)`
* ESR: `transhipment_policy='E', ordering_policy='R'` and set `base_stock_vals=(WH, SA, SB)`
* CESR: `transhipment_policy='E', ordering_policy='C'` and set `base_stock_vals=(WH, SA, SB), order_cap=(SA,SB)`
* LA: `transhipment_policy='L'` 
* OSA: `transhipment_policy='O'` 
