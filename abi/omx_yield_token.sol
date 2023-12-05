/**
 * This file was automatically generated by Stylus and represents a Rust program.
 * For more information, please see [The Stylus SDK](https://github.com/OffchainLabs/stylus-sdk-rs).
 */

interface IYieldToken {
    function init(string calldata name, string calldata symbol, address minter, uint256 initial_supply) external;

    function setGov(address gov) external;

    function setInfo(string calldata name, string calldata symbol) external;

    function setYieldTrackers(address[] memory yield_trackers) external;

    function addAdmin(address account) external;

    function removeAdmin(address account) external;

    function setInWhitelistMode(bool in_whitelist_mode) external;

    function setWhitelistedHandler(address handler, bool is_whitelisted) external;

    function addNonStakingAccount(address account) external;

    function removeNonStakingAccount(address account) external;

    function recoverClaim(address account, address receiver) external;

    function claim(address receiver) external;

    function totalStaked() external view returns (uint256);

    function balanceOf(address account) external view returns (uint256);

    function stakedBalance(address account) external view returns (uint256);

    function transfer(address recipient, uint256 amount) external returns (bool);

    function allowance(address owner, address spender) external view returns (uint256);

    function approve(address spender, uint256 amount) external returns (bool);

    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

    function setMinter(address minter, bool is_active) external;

    function mint(address account, uint256 amount) external;

    function burn(address account, uint256 amount) external;
}
