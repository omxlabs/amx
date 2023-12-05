extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IPositionsDecreaseManager {
        function init(address gov, address vault, address funding_rate_manager, address decrease_router, address positions_manager, address positions_liquidation_manager, address positions_manager_utils) external;

        function setGov(address gov) external;

        function decreasePosition(address account, address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long, address receiver) external returns (uint256);
    }
}
