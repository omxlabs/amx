/**
 * This file was automatically generated by Stylus and represents a Rust program.
 * For more information, please see [The Stylus SDK](https://github.com/OffchainLabs/stylus-sdk-rs).
 */

interface IOrderbookSwap {
    function init(address gov, address swap_router) external;

    function getCurrentIndex(address account) external view returns (uint256);

    function createSwapOrder(address token_in, address token_out, uint256 amount_in, uint256 min_out, uint256 trigger_ratio, bool trigger_above_threshold, uint256 execution_fee) external payable;

    function cancelSwapOrder(uint256 order_index) external;

    function getSwapOrder(address account, uint256 order_index) external view returns (address, address, address, uint256, uint256, uint256, bool, uint256);
}
