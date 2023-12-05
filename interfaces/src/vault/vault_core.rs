extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IVault {
        function init(address gov, address swap_manager, address positions_manager, address positions_increase_manager, address positions_decrease_manager, address positions_liquidation_manager, address positions_manager_utils, address price_feed) external;

        function isStable(address token) external view returns (bool);

        function isShortable(address token) external view returns (bool);

        function isWhitelisted(address token) external view returns (bool);

        function getTokenDecimals(address token) external view returns (uint8);

        function tokenWeight(address token) external view returns (uint256);

        function totalTokenWeights() external view returns (uint256);

        function reservedAmount(address token) external view returns (uint256);

        function minProfitBasisPoint(address token) external view returns (uint256);

        function allWhitelistedTokensLength() external view returns (uint256);

        function allWhitelistedTokens(uint64 index) external view returns (address);

        function poolAmount(address token) external view returns (uint256);

        function transferIn(address token) external returns (uint256);

        function transferOut(address token, uint256 amount, address receiver) external;

        function decreasePoolAmount(address token, uint256 amount) external;

        function increasePoolAmount(address token, uint256 amount) external;

        function updateTokenBalance(address token) external;

        function getPrice(address token) external view returns (uint256);

        function setTokenConfig(address token, uint8 token_decimals, uint256 token_weight, uint256 min_profit_basis_points, bool is_stable, bool is_shortable) external;

        function clearTokenConfig(address token) external;

        function directPoolDeposit(address token) external;

        function increaseReservedAmount(address token, uint256 amount) external;

        function decreaseReservedAmount(address token, uint256 amount) external;
    }
}
