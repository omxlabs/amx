#[allow(clippy::too_many_arguments)]
pub mod fee_manager;
#[allow(clippy::too_many_arguments)]
pub mod funding_rate_manager;
pub mod position;
#[allow(clippy::too_many_arguments)]
pub mod positions_decrease_manager;
#[allow(clippy::too_many_arguments)]
pub mod positions_increase_manager;
#[allow(clippy::too_many_arguments)]
pub mod positions_liquidation_manager;
#[allow(clippy::too_many_arguments)]
pub mod positions_manager;
#[allow(clippy::too_many_arguments)]
pub mod positions_manager_utils;
#[allow(clippy::too_many_arguments)]
pub mod shorts_tracker;
#[allow(clippy::too_many_arguments)]
pub mod swap_manager;
#[allow(clippy::too_many_arguments)]
pub mod vault_core;
#[allow(clippy::too_many_arguments)]
pub mod vault_utils;

pub use fee_manager::*;
pub use funding_rate_manager::*;
pub use position::*;
pub use positions_decrease_manager::*;
pub use positions_increase_manager::*;
pub use positions_liquidation_manager::*;
pub use positions_manager::*;
pub use positions_manager_utils::*;
pub use shorts_tracker::*;
pub use swap_manager::*;
pub use vault_core::*;
pub use vault_utils::*;

extern crate alloc;

use alloy_sol_types::sol;

sol! {
    event BuyUSDO(address account, address token, uint256 token_amount, uint256 usdo_amount, uint256 fee_basis_points);
    event SellUSDO(address account, address token, uint256 usdo_amount, uint256 token_amount, uint256 fee_basis_points);
    event Swap(address account, address token_in, address token_out, uint256 amount_in, uint256 amount_out, uint256 amount_out_after_fees, uint256 fee_basis_points);

    event IncreasePosition(
        address account,
        address collateral_token,
        address index_token,
        uint256 collateral_delta,
        uint256 size_delta,
        bool is_long,
        uint256 price,
        uint256 fee
    );
    event DecreasePosition(
        address account,
        address collateral_token,
        address index_token,
        uint256 collateral_delta,
        uint256 size_delta,
        bool is_long,
        uint256 price,
        uint256 fee,
        uint256 usd_out
    );
    event LiquidatePosition(
        address account,
        address collateral_token,
        address index_token,
        bool is_long,
        uint256 size,
        uint256 collateral,
        uint256 reserve_amount,
        int256 realised_pnl,
        uint256 mark_price
    );
    event UpdatePosition(
        uint256 size,
        uint256 collateral,
        uint256 average_price,
        uint256 entry_funding_rate,
        uint256 reserve_amount,
        int256 realised_pnl,
        uint256 mark_price
    );
    event ClosePosition(
        uint256 size,
        uint256 collateral,
        uint256 average_price,
        uint256 entry_funding_rate,
        uint256 reserve_amount,
        int256 realised_pnl
    );

    event UpdateFundingRate(address token, uint256 funding_rate);
    event UpdatePnl(
        address account,
        address collateral_token,
        address index_token,
        bool is_long,
        bool has_profit,
        uint256 delta
    );

    event CollectSwapFees(address token, uint256 fee_usd, uint256 fee_tokens);
    event CollectMarginFees(address token, uint256 fee_usd, uint256 fee_tokens);

    event DecreasePoolAmount(address token, uint256 amount);
    event IncreaseUsdoAmount(address token, uint256 amount);
    event DecreaseUsdoAmount(address token, uint256 amount);
    event IncreaseReservedAmount(address token, uint256 amount);
    event DecreaseReservedAmount(address token, uint256 amount);

    event IncreasePoolAmount(address token, uint256 amount);
    event IncreaseGuaranteedUsd(address token, uint256 amount);
    event DirectPoolDeposit(address token, uint256 amount);
    event DecreaseGuaranteedUsd(address token, uint256 amount);
}

pub fn validate(expr: bool, err: VaultError) -> Result<(), Vec<u8>> {
    if expr {
        Ok(())
    } else {
        Err(err.into())
    }
}
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VaultError {
    NotInitialized,
    Forbidden,
    ManagerOnly,
    GasPriceTooHigh,
    MaxFeeBasisPointsExceeded,
    MaxUsdoAmountExceeded,
    ZeroUsdoAmount,
    SwapNotEnabled,
    SameToken,
    PoolLessThenBuffer,
    PoolLessThenReserved,
    PoolExceeded,
    PoolExceededBalance,
    CollateralNotIndex,
    CollateralNotWhitelisted,
    CollateralIsStable,
    CollateralNotStable,
    IndexIsStable,
    IndexNotShortable,
    CollateralNotZero,
    CollateralLessThenFees,
    SizeLessThenCollateral,
    AveragePriceZero,
    ZeroAmount,
    ZeroCollateral,
    ZeroSize,
    ZeroRedemptionAmount,
    SizeLessThenDelta,
    CollateralLessThenDelta,
    PnlOverflow,
    LossesExceedCollateral,
    MaxShortsExceeded,
    FeesExceedCollateral,
    FeesExceedAmountOut,
    LiquidationFeesExceedCollateral,
    MaxLeverageExceeded,
    LiquidateValidPosition,
    AlreadyInitialized,
    MaxLeverageTooLow,
    LiquidationFeeUsdTooHigh,
    FundingIntervalTooLow,
    FundingRateFactorTooHigh,
    TokenNotWhitelisted,
}

impl From<VaultError> for Vec<u8> {
    fn from(err: VaultError) -> Vec<u8> {
        use VaultError as E;
        let err = match err {
            E::NotInitialized => "not initialized",
            E::Forbidden => "forbidden",
            E::SwapNotEnabled => "swap not enabled",
            E::ManagerOnly => "manager only",
            E::GasPriceTooHigh => "gas price is too high",
            E::MaxUsdoAmountExceeded => "max USDO amount exceeded",
            E::PoolLessThenBuffer => "pool_amount < buffer",
            E::PoolLessThenReserved => "pool_amount < reserved",
            E::PoolExceeded => "pool_amount exceeded",
            E::MaxShortsExceeded => "max shorts exceeded",
            E::PoolExceededBalance => "pool_amount exceeded balance",
            E::CollateralNotIndex => "collateral token should be same as index for long position",
            E::CollateralNotWhitelisted => "collateral token is not whitelisted",
            E::CollateralIsStable => "collateral token is stable",
            E::CollateralNotStable => "collateral token is not stable",
            E::IndexIsStable => "index token is stable",
            E::IndexNotShortable => "index token is not shortable",
            E::CollateralNotZero => "collateral is not zero",
            E::CollateralLessThenFees => "collateral is less then fees",
            E::SizeLessThenCollateral => "size is less then collateral",
            E::AveragePriceZero => "average price is zero",
            E::ZeroAmount => "zero amount",
            E::ZeroCollateral => "zero collateral",
            E::ZeroUsdoAmount => "zero USDO amount",
            E::ZeroSize => "zero size",
            E::ZeroRedemptionAmount => "zero redemption amount",
            E::SizeLessThenDelta => "size is less then delta",
            E::CollateralLessThenDelta => "collateral is less then delta",
            E::PnlOverflow => "realised pnl overflow",
            E::LossesExceedCollateral => "losses exceed collateral",
            E::FeesExceedCollateral => "fees exceed collateral",
            E::FeesExceedAmountOut => "fees exceed amount out",
            E::LiquidationFeesExceedCollateral => "liquidation fees exceed collateral",
            E::MaxLeverageExceeded => "max leverage exceeded",
            E::LiquidateValidPosition => "liquidating valid position",
            E::AlreadyInitialized => "already initialized",
            E::MaxLeverageTooLow => "max leverage too low",
            E::MaxFeeBasisPointsExceeded => "max fee basis points exceeded",
            E::LiquidationFeeUsdTooHigh => "liquidation fee usd too high",
            E::FundingIntervalTooLow => "funding interval too low",
            E::FundingRateFactorTooHigh => "funding rate factor too high",
            E::TokenNotWhitelisted => "token not whitelisted",
            E::SameToken => "token in and token out are the same",
        };

        format!("Vault: {err}").into()
    }
}
