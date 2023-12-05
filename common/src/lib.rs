use ruint::{aliases::U256, uint};

pub use safe_math::*;

pub mod call_context;
pub mod reentrant;
pub mod safe_math;

pub const SECOND: U256 = uint!(1_U256);
pub const MINUTE: U256 = uint!(60_U256);
pub const HOUR: U256 = uint!(3600_U256);
pub const DAY: U256 = uint!(86400_U256);
pub const WEEK: U256 = uint!(604800_U256);
pub const YEAR: U256 = uint!(31536000_U256);

pub const PRICE_PRECISION: U256 = uint!(1000000000000000000000000000000_U256);
pub const ONE_USD: U256 = PRICE_PRECISION;
pub const BASIS_POINTS_DIVISOR: U256 = uint!(10000_U256);
pub const MAX_SPREAD_BASIS_POINTS: U256 = uint!(50_U256);
pub const MAX_ADJUSTMENT_BASIS_POINTS: U256 = uint!(20_U256);
pub const MAX_ADJUSTMENT_INTERVAL: U256 = uint!(7200_U256); // 2 hours
pub const FUNDING_RATE_PRECISION: U256 = uint!(1000000_U256);
pub const ETH_DECIMALS: u8 = 18;
pub const USDO_DECIMALS: u8 = 18;
pub const OLP_PRECISION: U256 = uint!(1000000000000000000_U256);

pub const TAX_BASIS_POINTS: U256 = uint!(50_U256); // 0.5%
pub const STABLE_TAX_BASIS_POINTS: U256 = uint!(20_U256); // 0.2%
pub const MINT_BURN_FEE_BASIS_POINTS: U256 = uint!(30_U256); // 0.3%
pub const SWAP_FEE_BASIS_POINTS: U256 = uint!(30_U256); // 0.3%
pub const STABLE_SWAP_FEE_BASIS_POINTS: U256 = uint!(4_U256); // 0.04%
pub const MARGIN_FEE_BASIS_POINTS: U256 = uint!(10_U256); // 0.1%

pub const FUNDING_INTERVAL: U256 = HOUR;
pub const FUNDING_RATE_FACTOR: U256 = uint!(100_U256);
pub const STABLE_FUNDING_RATE_FACTOR: U256 = uint!(100_U256);
pub const LIQUIDATION_FEE_USD: U256 = uint!(50000000000000000000000000000_U256); // 0.05 USD

pub const MAX_LEVERAGE: U256 = uint!(500000_U256); // 50x

pub const LIQUIDATION_STATE_NORMAL: U256 = uint!(0_U256);

pub const MIN_EXECUTION_FEE: U256 = uint!(1000_U256);
// pub const MIN_EXECUTION_FEE: U256 = uint!(10000000000000000_U256);
pub const MIN_PURCHASE_TOKEN_AMOUNT_US: U256 = PRICE_PRECISION;

// pub const OLP_MANAGER_COOLDOWN: U256 = uint!(900_U256); // 15 minutes
pub const OLP_MANAGER_COOLDOWN: U256 = uint!(86400_U256); // 24 hours
pub const OLP_MANAGER_AUM_ADDITION: U256 = uint!(0_U256);
pub const OLP_MANAGER_AUM_DEDUCTION: U256 = uint!(0_U256);

pub const BONUS_DURATION: U256 = YEAR;

pub const DISTRIBUTION_INTERVAL: U256 = HOUR;
