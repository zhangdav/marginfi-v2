use fixed::types::I80F48;

use crate::state::marginfi_group::WrappedI80F48;

pub fn wrapped_i80f48_to_f64(n: WrappedI80F48) -> f64 {
    let as_i80: I80F48 = n.into();
    let as_f64: f64 = as_i80.to_num();
    as_f64
}
