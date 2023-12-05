extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IShortsTracker {
        function isGlobalShortDataReady() external view returns (bool);
        function globalShortAveragePrices(address token) external view returns (uint256);
        function getNextGlobalShortData(
            address account,
            address collateral_token,
            address index_token,
            uint256 next_price,
            uint256 size_delta,
            bool is_increase
        ) external view returns (uint256, uint256);
        function updateGlobalShortData(
            address account,
            address collateral_token,
            address index_token,
            bool is_long,
            uint256 size_delta,
            uint256 mark_price,
            bool is_increase
        ) external;
        function setIsGlobalShortDataReady(bool value) external;
        function setInitData(address[] calldata tokens, uint256[] calldata average_prices) external;
    }

}
