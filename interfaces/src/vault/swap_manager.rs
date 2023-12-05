extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface ISwapManager {
        function init(address gov, address usdo, address vault, address fee_manager, address funding_rate_manager) external;

        function setManager(address account, bool is_manager) external;

        function setInManagerMode(bool in_manager_mode) external;

        function usdoAmount(address token) external view returns (uint256);

        function buyUsdo(address token, address receiver) external returns (uint256);

        function sellUsdo(address token, address receiver) external returns (uint256);

        function swap(address token_in, address token_out, address receiver) external returns (uint256);
    }
}
