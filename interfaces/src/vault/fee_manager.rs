extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IFeeManager {
        function init(address gov, address usdo, address vault, address funding_rate_manager, address swap_manager, address positions_manager, address positions_manager_utils, address positions_increase_manager, address positions_decrease_manager, address positions_liquidation_manager) external;

        function setGov(address gov) external;

        function increaseFeeReserves(address token, uint256 amount) external;

        function getFeeReserve(address token) external view returns (uint256);

        function collectSwapFees(address token, uint256 amount, uint256 fee_basis_points) external returns (uint256);

        function getPositionFee(uint256 size_delta) external view returns (uint256);

        function getFundingFee(address collateral_token, uint256 size, uint256 entry_funding_rate) external view returns (uint256);

        function collectMarginFees(address collateral_token, uint256 size_delta, uint256 size, uint256 entry_funding_rate) external returns (uint256);

        function withdrawFees(address token, address receiver) external returns (uint256);

        function getSwapFeeBasisPoints(address token_in, address token_out) external view returns (uint256);

        function getTargetUsdoAmount(address token) external view returns (uint256);
    }
}
