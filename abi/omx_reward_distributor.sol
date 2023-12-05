/**
 * This file was automatically generated by Stylus and represents a Rust program.
 * For more information, please see [The Stylus SDK](https://github.com/OffchainLabs/stylus-sdk-rs).
 */

interface IRewardDistributor {
    function init(address gov, address reward_token, address reward_tracker, address reward_tracker_staking) external;

    function setGov(address gov) external;

    function setAdmin(address admin) external;

    function updateLastDistributionTime() external;

    function setTokensPerInterval(uint256 amount) external;

    function pendingRewards() external view returns (uint256);

    function distribute() external returns (uint256);

    function rewardToken() external view returns (address);

    function tokensPerInterval() external view returns (uint256);
}