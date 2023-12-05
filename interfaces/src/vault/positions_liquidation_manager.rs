extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IPositionsLiquidationManager {
        function init(address gov, address vault, address vault_utils, address fee_manager, address funding_rate_manager, address positions_manager, address positions_manager_utils, address positions_decrease_manager) external;

        function setGov(address gov) external;

        function setLiquidator(address liquidator, bool is_active) external;

        function liquidatePosition(address account, address collateral_token, address index_token, bool is_long, address fee_receiver) external;
    }
}
