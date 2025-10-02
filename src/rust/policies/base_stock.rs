use std::cmp::max;

// Implement a base-stock policy for rust
// Note there is a lead-time of 1 for the store and warehouse.
pub fn regular_base_stock(state: (usize, usize, usize), warehouse_bs: usize, store_bs: (usize, usize)) -> (usize, usize, usize) {
    // Remember store_bs.0 is the first store and store_bs.1 is the second stores order up to
    let mut desired_sa = max(store_bs.0 as isize - state.1 as isize, 0) as usize;
    let mut desired_sb = max(store_bs.1 as isize - state.2 as isize, 0) as usize;

    let wh_order: usize = max(warehouse_bs as isize - max(state.0 as isize - desired_sa as isize - desired_sb as isize,0) as isize, 0) as usize;

    // if the desired is more than the warehouse level then we need to allocate
    if state.0 < desired_sa + desired_sb {
        (desired_sa, desired_sb) = allocate_stock(state.0, desired_sa, desired_sb);
    } 
    (wh_order, desired_sa, desired_sb)
}

// pub fn echelon_base_stock() -> Result<u64, ()> {
//     // Placeholder for the echelon base stock policy
//     unimplemented!("Echelon base stock policy is not implemented yet");
// }

pub fn allocate_stock(wh_state: usize, sa_request: usize, sb_request: usize) -> (usize, usize) {
    let mut sa_alloc = 0;
    let mut sb_alloc = 0;
    let mut wh_available = wh_state;

    while wh_available > 0 {
        let current_sa_request = sa_request - sa_alloc;
        let current_sb_request = sb_request - sb_alloc;
        if current_sa_request > current_sb_request {
            sa_alloc += 1;
        } else {
            sb_alloc += 1;
        }
        wh_available -= 1;
    }
    (sa_alloc,sb_alloc)
}