extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IPositionsManagerUtils {
        function init(address positions_manager, address positions_decrease_manager, address vault, address fee_manager, address vault_utils) external;

        function reduceCollateral(address account, address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long) external returns (uint256, uint256);

        function validateLiquidation(address account, address collateral_token, address index_token, bool is_long, bool raise) external view returns (uint256, uint256);

        function getNextAveragePrice(address index_token, uint256 size, uint256 average_price, bool is_long, uint256 next_price, uint256 size_delta, uint256 last_increased_time) external view returns (uint256);
    }
}
