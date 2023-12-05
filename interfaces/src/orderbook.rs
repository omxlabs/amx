extern crate alloc;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    event CreateIncreaseOrder(
        address indexed account,
        uint256 order_index,
        uint256 collateral_amount,
        address collateral_token,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event CancelIncreaseOrder(
        address indexed account,
        uint256 order_index,
        uint256 collateral_amount,
        address collateral_token,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event ExecuteIncreaseOrder(
        address indexed account,
        uint256 order_index,
        uint256 collateral_amount,
        address collateral_token,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold,
        uint256 execution_fee,
        uint256 execution_price
    );
    event UpdateIncreaseOrder(
        address indexed account,
        uint256 order_index,
        address collateral_token,
        address index_token,
        bool is_long,
        uint256 size_delta,
        uint256 trigger_price,
        bool trigger_above_threshold
    );
    event CreateDecreaseOrder(
        address indexed account,
        uint256 order_index,
        address collateral_token,
        uint256 collateral_delta,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event CancelDecreaseOrder(
        address indexed account,
        uint256 order_index,
        address collateral_token,
        uint256 collateral_delta,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event ExecuteDecreaseOrder(
        address indexed account,
        uint256 order_index,
        address collateral_token,
        uint256 collateral_delta,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold,
        uint256 execution_fee,
        uint256 execution_price
    );
    event UpdateDecreaseOrder(
        address indexed account,
        uint256 order_index,
        address collateral_token,
        uint256 collateral_delta,
        address index_token,
        uint256 size_delta,
        bool is_long,
        uint256 trigger_price,
        bool trigger_above_threshold
    );
    event CreateSwapOrder(
        address indexed account,
        uint256 order_index,
        address token_in,
        address token_out,
        uint256 amount_in,
        uint256 min_out,
        uint256 trigger_ratio,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event CancelSwapOrder(
        address indexed account,
        uint256 order_index,
        address token_in,
        address token_out,
        uint256 amount_in,
        uint256 min_out,
        uint256 trigger_ratio,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event UpdateSwapOrder(
        address indexed account,
        uint256 order_index,
        address token_in,
        address token_out,
        uint256 amount_in,
        uint256 min_out,
        uint256 trigger_ratio,
        bool trigger_above_threshold,
        uint256 execution_fee
    );
    event ExecuteSwapOrder(
        address indexed account,
        uint256 order_index,
        address token_in,
        address token_out,
        uint256 amount_in,
        uint256 min_out,
        uint256 amount_out,
        uint256 trigger_ratio,
        bool trigger_above_threshold,
        uint256 execution_fee
    );

    event UpdateMinExecutionFee(uint256 min_execution_fee);
    event UpdateMinPurchaseTokenAmountUsd(uint256 min_purchase_token_amount_usd);
    event UpdateGov(address gov);

    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
    error ZeroAmountIn();
    error ZeroCollateralAmount();
    error InsufficientExecutionFee(uint256 min_execution_fee);
    error CanNotWrapToken(address token);
    error IncorrectValueTransferred(uint256 expected);
    error IncorrectFeeTransferred(uint256 expected);
    error OrderNotFound(uint256 order_index);
    error InvalidExecutionPrice(uint256 execution_price, uint256 trigger_price, bool trigger_above_threshold);
}

/// - account
/// - token_in
/// - token_out
/// - amount_in
/// - min_out
/// - trigger_ratio
/// - trigger_above_threshold
/// - execution_fee
pub type RawSwapOrder = (Address, Address, Address, U256, U256, U256, bool, U256);

/// - account
/// - collateral_amount
/// - collateral_token
/// - index_token
/// - size_delta
/// - is_long
/// - trigger_price
/// - trigger_above_threshold
/// - execution_fee
pub type RawIncreaseOrder = (
    Address,
    U256,
    Address,
    Address,
    U256,
    bool,
    U256,
    bool,
    U256,
);

sol_interface! {
    interface IOrderbookSwap {
        function init(address gov, address swap_router) external;

        function getCurrentIndex(address account) external view returns (uint256);

        function createSwapOrder(address token_in, address token_out, uint256 amount_in, uint256 min_out, uint256 trigger_ratio, bool trigger_above_threshold, uint256 execution_fee) external payable;

        function cancelSwapOrder(uint256 order_index) external;

        function getSwapOrder(address account, uint256 order_index) external view returns (address, address, address, uint256, uint256, uint256, bool, uint256);
    }
}

sol_interface! {
    interface IOrderbookIncrease {
        function init(address gov, address increase_router) external;

        function getCurrentIndex(address account) external view returns (uint256);

        function createIncreaseOrder(uint256 collateral_amount, address collateral_token, address index_token, uint256 size_delta, bool is_long, uint256 trigger_price, bool trigger_above_threshold, uint256 execution_fee) external payable;

        function cancelIncreaseOrder(uint256 order_index) external;

        function getIncreaseOrder(address account, uint256 order_index) external view returns (address, uint256, address, address, uint256, bool, uint256, bool, uint256);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderbookError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    ZeroAmountIn,
    ZeroCollateralAmount,
    InsufficientExecutionFee {
        min_execution_fee: U256,
    },
    CanNotWrapToken {
        token: Address,
    },
    IncorrectValueTransferred {
        expected: U256,
    },
    IncorrectFeeTransferred {
        expected: U256,
    },
    OrderNotFound {
        order_index: U256,
    },
    InvalidExecutionPrice {
        execution_price: U256,
        trigger_price: U256,
        trigger_above_threshold: bool,
    },
}

impl From<OrderbookError> for Vec<u8> {
    fn from(err: OrderbookError) -> Vec<u8> {
        use OrderbookError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::CanNotWrapToken { token } => CanNotWrapToken { token }.encode(),
            E::IncorrectValueTransferred { expected } => {
                IncorrectValueTransferred { expected }.encode()
            }
            E::IncorrectFeeTransferred { expected } => {
                IncorrectFeeTransferred { expected }.encode()
            }
            E::OrderNotFound { order_index } => OrderNotFound { order_index }.encode(),
            E::InvalidExecutionPrice {
                execution_price,
                trigger_price,
                trigger_above_threshold,
            } => InvalidExecutionPrice {
                execution_price,
                trigger_price,
                trigger_above_threshold,
            }
            .encode(),
            E::ZeroAmountIn => ZeroAmountIn {}.encode(),
            E::ZeroCollateralAmount => ZeroCollateralAmount {}.encode(),
            E::InsufficientExecutionFee { min_execution_fee } => {
                InsufficientExecutionFee { min_execution_fee }.encode()
            }
        }
    }
}
