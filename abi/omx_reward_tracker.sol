/**
 * This file was automatically generated by Stylus and represents a Rust program.
 * For more information, please see [The Stylus SDK](https://github.com/OffchainLabs/stylus-sdk-rs).
 */

interface IRewardTracker {
    function init(address gov, address distributor, address reward_tracker_staking, string calldata name, string calldata symbol) external;

    function totalSupply() external view returns (uint256);

    function isHandler(address account) external view returns (bool);

    function setGov(address gov) external;

    function setInPrivateTransferMode(bool in_private_transfer_mode) external;

    function setHandler(address handler, bool is_active) external;

    function balanceOf(address account) external view returns (uint256);

    function transfer(address recipient, uint256 amount) external returns (bool);

    function allowance(address owner, address spender) external view returns (uint256);

    function approve(address spender, uint256 amount) external returns (bool);

    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

    function tokensPerInterval() external view returns (uint256);

    function rewardToken() external view returns (address);

    function mintInternal(address account, uint256 amount) external;

    function burnInternal(address account, uint256 amount) external;
}
