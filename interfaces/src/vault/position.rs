use alloy_primitives::{keccak256, Address, FixedBytes, I256, U256};
use stylus_sdk::hex::ToHex;

use crate::vault::{validate, VaultError};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub size: U256,
    pub collateral: U256,
    pub average_price: U256,
    pub entry_funding_rate: U256,
    pub reserve_amount: U256,
    pub realised_pnl: I256,
    pub last_increased_time: U256,
}

/// - size
/// - collateral
/// - average_price
/// - entry_funding_rate
/// - reserve_amount
/// - realised_pnl
/// - last_increased_time
pub type RawPositionData = (U256, U256, U256, U256, U256, I256, U256);

#[inline]
pub fn pos_size(pos: RawPositionData) -> U256 {
    pos.0
}

#[inline]
pub fn pos_collateral(pos: RawPositionData) -> U256 {
    pos.1
}

#[inline]
pub fn pos_average_price(pos: RawPositionData) -> U256 {
    pos.2
}

#[inline]
pub fn pos_entry_funding_rate(pos: RawPositionData) -> U256 {
    pos.3
}

#[inline]
pub fn pos_reserve_amount(pos: RawPositionData) -> U256 {
    pos.4
}

#[inline]
pub fn pos_realised_pnl(pos: RawPositionData) -> I256 {
    pos.5
}

#[inline]
pub fn pos_last_increased_time(pos: RawPositionData) -> U256 {
    pos.6
}

impl From<RawPositionData> for Position {
    fn from(value: RawPositionData) -> Self {
        Self {
            size: value.0,
            collateral: value.1,
            average_price: value.2,
            entry_funding_rate: value.3,
            reserve_amount: value.4,
            realised_pnl: value.5,
            last_increased_time: value.6,
        }
    }
}

pub fn validate_position(size: U256, collateral: U256) -> Result<(), Vec<u8>> {
    if size == U256::ZERO {
        validate(collateral == U256::ZERO, VaultError::CollateralNotZero)?;
        return Ok(());
    }

    validate(size >= collateral, VaultError::SizeLessThenCollateral)?;

    Ok(())
}

pub fn get_position_key(
    account: Address,
    collateral_token: Address,
    index_token: Address,
    is_long: bool,
) -> FixedBytes<32> {
    let data: Vec<_> = [
        account.encode_hex(),
        collateral_token.encode_hex(),
        index_token.encode_hex(),
        vec![is_long as u8 as char],
    ]
    .into_iter()
    .flatten()
    .map(|v| v as u8)
    .collect();

    keccak256(data)
}
