extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IPositionsIncreaseManager {
        function init(address gov, address vault, address vault_utils, address fee_manager, address funding_rate_manager, address increase_router, address positions_manager, address positions_manager_utils) external;

        function setGov(address gov) external;

        function increasePosition(address account, address collateral_token, address index_token, uint256 size_delta, bool is_long) external;
    }
}
