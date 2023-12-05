extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IFundingRateManager {
        function init(address gov, address vault) external;

        function setGov(address gov) external;

        function cumulativeFundingRate(address token) external view returns (uint256);

        function getNextFundingRate(address token) external view returns (uint256);

        function updateCumulativeFundingRate(address collateral_token) external;
    }
}
