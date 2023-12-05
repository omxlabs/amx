extern crate alloc;

use alloy_primitives::U256;
use alloy_sol_types::sol;
use ruint::uint;
use stylus_sdk::stylus_proc::sol_interface;

pub const MAX_PRICE_AGE: U256 = uint!(60_U256);

sol! {
    event Claim(address receiver, uint256 amount);
}

sol_interface! {
    interface IVaultPriceFeed {
        function init(address gov, address pyth, uint256 max_strict_price_deviation) external;

        function setGov(address gov) external;

        function setAdjustment(address token, bool is_additive, uint256 adjustment_bps) external;

        function setPyth(address pyth) external;

        function setSpreadBasisPoints(address token, uint256 spread_basis_points) external;

        function setMaxStrictPriceDeviation(uint256 max_strict_price_deviation) external;

        function setTokenConfig(address token, bytes32 price_feed, bool is_strict_stable) external;

        function getPrice(address token, bool maximize) external view returns (uint256);

        function getPriceV1(address token, bool maximize) external view returns (uint256);

        function getPrimaryPrice(address token, bool maximise) external view returns (uint256);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VaultPriceFeedError {
    AlreadyInitialized,
    PriceOverflow,
    Forbidden,
    AdjustmentFrequencyExceeded,
    InvalidAdjustmentBps,
    InvalidSpreadBasisPoints,
    InvalidPriceSampleSpace,
    InvalidPriceFeed,
    InvalidPrice,
    CouldNotFetchPrice,
    PriceTooOld,
}

impl From<VaultPriceFeedError> for Vec<u8> {
    fn from(err: VaultPriceFeedError) -> Vec<u8> {
        use VaultPriceFeedError as E;
        let err = match err {
            E::AlreadyInitialized => "already initialized",
            E::PriceOverflow => "price overflow",
            E::Forbidden => "forbidden",
            E::AdjustmentFrequencyExceeded => "adjustment frequency exceeded",
            E::InvalidAdjustmentBps => "invalid adjustment_bps",
            E::InvalidSpreadBasisPoints => "invalid spread_basis_points",
            E::InvalidPriceSampleSpace => "invalid price_sample_space",
            E::InvalidPriceFeed => "invalid price feed",
            E::InvalidPrice => "invalid price",
            E::CouldNotFetchPrice => "could not fetch price",
            E::PriceTooOld => "price too old",
        };

        format!("VaultPriceFeed: {err}").into()
    }
}
