extern crate alloc;

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    event AddLiquidity(
        address account,
        address token,
        uint256 amount,
        uint256 aum_in_usdo,
        uint256 olp_supply,
        uint256 usdo_amount,
        uint256 mint_amount
    );

    event RemoveLiquidity(
        address account,
        address token,
        uint256 olp_amount,
        uint256 aum_in_usdo,
        uint256 olp_supply,
        uint256 usdo_amount,
        uint256 amount_out
    );


    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
    error OlpZeroAmount();
    error TokenZeroAmount();
    error CooldownNotPassed();
    error ActionNotEnabled();
    error WeightToHigh(uint256 max_weight);
    error InsufficientOutput(uint256 amount_out);
    error InsufficientOlpOutput(uint256 amount_out);
    error InsufficientUsdoOutput(uint256 amount_out);
    error InvalidWeight(uint256 max_weight);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OlpManagerError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    OlpZeroAmount,
    TokenZeroAmount,
    CooldownNotPassed,
    ActionNotEnabled,
    WeightToHigh { max_weight: U256 },
    InsufficientOutput { amount_out: U256 },
    InsufficientOlpOutput { amount_out: U256 },
    InsufficientUsdoOutput { amount_out: U256 },
    InvalidWeight { max_weight: U256 },
}

impl From<OlpManagerError> for Vec<u8> {
    fn from(err: OlpManagerError) -> Vec<u8> {
        use OlpManagerError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::OlpZeroAmount => OlpZeroAmount {}.encode(),
            E::TokenZeroAmount => TokenZeroAmount {}.encode(),
            E::CooldownNotPassed => CooldownNotPassed {}.encode(),
            E::ActionNotEnabled => ActionNotEnabled {}.encode(),
            E::WeightToHigh { max_weight } => WeightToHigh { max_weight }.encode(),
            E::InsufficientOutput { amount_out } => InsufficientOutput { amount_out }.encode(),
            E::InsufficientOlpOutput { amount_out } => {
                InsufficientOlpOutput { amount_out }.encode()
            }
            E::InsufficientUsdoOutput { amount_out } => {
                InsufficientUsdoOutput { amount_out }.encode()
            }
            E::InvalidWeight { max_weight } => InvalidWeight { max_weight }.encode(),
        }
    }
}

sol_interface! {
    interface IOlpManager {
        function init(address gov, address olp_manager_utils, address vault, address swap_manager, address positions_manager, address shorts_tracker, address usdo, address olp) external;

        function setGov(address gov) external;

        function setInPrivateMode(bool in_private_mode) external;

        function setHandler(address handler, bool is_active) external;

        function addLiquidity(address token, uint256 amount, uint256 min_usdo, uint256 min_olp) external returns (uint256);

        function addLiquidityForAccount(address funding_account, address account, address token, uint256 amount, uint256 min_usdo, uint256 min_olp) external returns (uint256);

        function removeLiquidity(address token_out, uint256 olp_amount, uint256 min_amount, address recipient) external returns (uint256);

        function removeLiquidityForAccount(address account, address token_out, uint256 olp_amount, uint256 min_amount, address recipient) external returns (uint256);
    }
}

sol_interface! {
    interface IOlpManagerUtils {
        function init(address vault, address positions_manager, address shorts_tracker, address olp) external;

        function isHandler(address account) external view returns (bool);

        function setShortsTrackerAveragePriceWeight(uint256 weight) external;

        function getPrice() external view returns (uint256);

        function getAumInUsdo() external view returns (uint256);

        function getAum() external view returns (uint256);

        function getGlobalShortDelta(address token, uint256 price, uint256 size) external view returns (uint256, bool);

        function getGlobalShortAveragePrice(address token) external view returns (uint256);
    }
}
