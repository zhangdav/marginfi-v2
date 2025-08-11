use crate::{state::marginfi_group::Bank, utils::wrapped_i80f48_to_f64};
use anchor_lang::prelude::*;

pub fn log_pool_info(bank: &Bank) {
    let conf = bank.config;
    msg!(
        "Asset weight init: {:?} maint: {:?}",
        wrapped_i80f48_to_f64(conf.asset_weight_init),
        wrapped_i80f48_to_f64(conf.asset_weight_maint)
    );
    msg!(
        "Liability weight init: {:?} maint: {:?}",
        wrapped_i80f48_to_f64(conf.liability_weight_init),
        wrapped_i80f48_to_f64(conf.liability_weight_maint)
    );
    msg!(
        "deposit limit: {:?} borrow limit: {:?} init limit: {:?}",
        conf.deposit_limit,
        conf.borrow_limit,
        conf.total_asset_value_init_limit
    );
    msg!(
        "op state: {:?} age: {:?} flags: {:?}",
        conf.oracle_max_confidence,
        conf.oracle_max_age as u8,
        bank.flags as u8
    );
    let interest = conf.interest_rate_config;
    msg!(
        "Insurance fixed: {:?} ir: {:?}",
        wrapped_i80f48_to_f64(interest.insurance_fee_fixed_apr),
        wrapped_i80f48_to_f64(interest.insurance_ir_fee)
    );
    msg!(
        "Group fixed: {:?} ir: {:?} origination: {:?}",
        wrapped_i80f48_to_f64(interest.protocol_fixed_fee_apr),
        wrapped_i80f48_to_f64(interest.protocol_ir_fee),
        wrapped_i80f48_to_f64(interest.protocol_origination_fee)
    );
    msg!(
        "Plateau: {:?} Optimal: {:?} Max: {:?}",
        wrapped_i80f48_to_f64(interest.plateau_interest_rate),
        wrapped_i80f48_to_f64(interest.optimal_utilization_rate),
        wrapped_i80f48_to_f64(interest.max_interest_rate)
    );
}
