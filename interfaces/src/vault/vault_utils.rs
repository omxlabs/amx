extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IVaultUtils {
        function init(address vault, uint256 min_profit_time) external;

        function getUtilization(address token) external view returns (uint256);

        function validateTokens(address collateral_token, address index_token, bool is_long) external view;

        function tokenToUsd(address token, uint256 token_amount) external view returns (uint256);

        function usdToToken(address token, uint256 usd_amount) external view returns (uint256);

        function getDelta(address index_token, uint256 size, uint256 average_price, bool is_long, uint256 last_increased_time) external view returns (bool, uint256);
    }
}
