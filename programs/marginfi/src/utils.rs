use crate::state::marginfi_group::WrappedI80F48;
use crate::MarginfiResult;
use anchor_lang::prelude::*;
use anchor_spl::{
    token::Token,
    token_2022::spl_token_2022::{
        self,
        extension::{
            transfer_fee::{TransferFee, TransferFeeConfig},
            BaseStateWithExtensions, StateWithExtensions,
        },
    },
};
use fixed::types::I80F48;

const ONE_IN_BASIS_POINTS: u128 = 10_000;

pub fn wrapped_i80f48_to_f64(n: WrappedI80F48) -> f64 {
    let as_i80: I80F48 = n.into();
    let as_f64: f64 = as_i80.to_num();
    as_f64
}

pub fn calculate_pre_fee_spl_deposit_amount(
    mint_ai: AccountInfo,
    post_fee_amount: u64,
    epoch: u64,
) -> MarginfiResult<u64> {
    // Determine whether it is a normal SPL Token (not the 2022 extended version)
    if mint_ai.owner.eq(&Token::id()) {
        return Ok(post_fee_amount);
    }

    let mint_data = mint_ai.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

    match mint.get_extension::<TransferFeeConfig>() {
        Ok(transfer_fee_config) => {
            let epoch_fee = transfer_fee_config.get_epoch_fee(epoch);
            let pre_fee_amount = calculate_pre_fee_amount(epoch_fee, post_fee_amount).unwrap();
            Ok(pre_fee_amount)
        }
        Err(_) => Ok(post_fee_amount),
    }
}

pub fn calculate_pre_fee_amount(transfer_fee: &TransferFee, post_fee_amount: u64) -> Option<u64> {
    let maximum_fee = u64::from(transfer_fee.maximum_fee);
    let transfer_fee_basis_points = u16::from(transfer_fee.transfer_fee_basis_points) as u128;
    match (transfer_fee_basis_points, post_fee_amount) {
        (0, _) => Some(post_fee_amount),
        (_, 0) => Some(0),
        (ONE_IN_BASIS_POINTS, _) => maximum_fee.checked_add(post_fee_amount),
        _ => {
            let numerator = (post_fee_amount as u128).checked_mul(ONE_IN_BASIS_POINTS)?;
            let denominator = ONE_IN_BASIS_POINTS.checked_sub(transfer_fee_basis_points)?;
            let raw_pre_fee_amount = ceil_div(numerator, denominator)?;

            if raw_pre_fee_amount.checked_sub(post_fee_amount as u128)? >= maximum_fee as u128 {
                post_fee_amount.checked_add(maximum_fee)
            } else {
                u64::try_from(raw_pre_fee_amount).ok()
            }
        }
    }
}

fn ceil_div(numerator: u128, denominator: u128) -> Option<u128> {
    numerator
        .checked_add(denominator)?
        .checked_sub(1)?
        .checked_div(denominator)
}
