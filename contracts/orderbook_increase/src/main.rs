#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{call_context::GetCallContext, safe_add, MIN_EXECUTION_FEE};
use omx_interfaces::{
    erc20::{safe_transfer, safe_transfer_from},
    orderbook::{CancelIncreaseOrder, CreateIncreaseOrder, OrderbookError, RawIncreaseOrder},
};
use stylus_sdk::{call::transfer_eth, contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct OrderbookIncrease {
        bool initialized;
        address gov;
        address increase_router;

        mapping (address => mapping(uint256 =>  address)) order_account;
        mapping (address => mapping(uint256 =>  uint256)) order_collateral_amount;
        mapping (address => mapping(uint256 =>  address)) order_collateral_token;
        mapping (address => mapping(uint256 =>  address)) order_index_token;
        mapping (address => mapping(uint256 =>  uint256)) order_size_delta;
        mapping (address => mapping(uint256 =>  bool)) order_is_long;
        mapping (address => mapping(uint256 =>  uint256)) order_trigger_price;
        mapping (address => mapping(uint256 =>  bool)) order_trigger_above_threshold;
        mapping (address => mapping(uint256 =>  uint256)) order_execution_fee;

        mapping (address => uint256) orders_index;
    }
}

impl OrderbookIncrease {
    fn only_initialized(&self) -> Result<(), OrderbookError> {
        if !self.initialized.get() {
            return Err(OrderbookError::AlreadyInitialized);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn create_order_internal(
        &mut self,
        account: Address,
        collateral_amount: U256,
        collateral_token: Address,
        index_token: Address,
        size_delta: U256,
        is_long: bool,
        trigger_price: U256,
        trigger_above_threshold: bool,
        execution_fee: U256,
    ) -> Result<(), Vec<u8>> {
        let order_index = self.orders_index.get(account);

        self.orders_index
            .insert(account, safe_add(order_index, U256::from(1))?);
        self.order_account
            .setter(account)
            .insert(order_index, account);
        self.order_collateral_amount
            .setter(account)
            .insert(order_index, collateral_amount);
        self.order_collateral_token
            .setter(account)
            .insert(order_index, collateral_token);
        self.order_index_token
            .setter(account)
            .insert(order_index, index_token);
        self.order_size_delta
            .setter(account)
            .insert(order_index, size_delta);
        self.order_is_long
            .setter(account)
            .insert(order_index, is_long);
        self.order_trigger_price
            .setter(account)
            .insert(order_index, trigger_price);
        self.order_trigger_above_threshold
            .setter(account)
            .insert(order_index, trigger_above_threshold);
        self.order_execution_fee
            .setter(account)
            .insert(order_index, execution_fee);

        evm::log(CreateIncreaseOrder {
            order_index,
            account,
            collateral_amount,
            collateral_token,
            index_token,
            size_delta,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        });

        Ok(())
    }
}

#[external]
impl OrderbookIncrease {
    pub fn init(&mut self, gov: Address, increase_router: Address) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(OrderbookError::AlreadyInitialized.into());
        }

        self.gov.set(gov);
        self.increase_router.set(increase_router);

        self.initialized.set(true);

        Ok(())
    }

    pub fn get_current_index(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.orders_index.get(account))
    }

    #[payable]
    #[allow(clippy::too_many_arguments)]
    pub fn create_increase_order(
        &mut self,
        collateral_amount: U256,
        collateral_token: Address,
        index_token: Address,
        size_delta: U256,
        is_long: bool,
        trigger_price: U256,
        trigger_above_threshold: bool,
        execution_fee: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        if collateral_amount == U256::ZERO {
            return Err(OrderbookError::ZeroCollateralAmount.into());
        }

        if execution_fee < MIN_EXECUTION_FEE {
            return Err(OrderbookError::InsufficientExecutionFee {
                min_execution_fee: MIN_EXECUTION_FEE,
            }
            .into());
        }

        if msg::value() != execution_fee {
            return Err(OrderbookError::IncorrectFeeTransferred {
                expected: execution_fee,
            }
            .into());
        }
        safe_transfer_from(
            self.ctx(),
            collateral_token,
            msg::sender(),
            contract::address(),
            collateral_amount,
        )?;

        self.create_order_internal(
            msg::sender(),
            collateral_amount,
            collateral_token,
            index_token,
            size_delta,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        )?;

        Ok(())
    }

    pub fn cancel_increase_order(&mut self, order_index: U256) -> Result<(), Vec<u8>> {
        let account = self.order_account.get(msg::sender()).get(order_index);
        if account.is_zero() {
            return Err(OrderbookError::OrderNotFound { order_index }.into());
        }

        let collateral_amount = self
            .order_collateral_amount
            .get(msg::sender())
            .get(order_index);
        let collateral_token = self
            .order_collateral_token
            .get(msg::sender())
            .get(order_index);
        let index_token = self.order_index_token.get(msg::sender()).get(order_index);
        let size_delta = self.order_size_delta.get(msg::sender()).get(order_index);
        let is_long = self.order_is_long.get(msg::sender()).get(order_index);
        let trigger_price = self.order_trigger_price.get(msg::sender()).get(order_index);
        let trigger_above_threshold = self
            .order_trigger_above_threshold
            .get(msg::sender())
            .get(order_index);
        let execution_fee = self.order_execution_fee.get(msg::sender()).get(order_index);

        self.order_account.setter(msg::sender()).delete(order_index);
        self.order_collateral_amount
            .setter(msg::sender())
            .delete(order_index);
        self.order_collateral_token
            .setter(msg::sender())
            .delete(order_index);
        self.order_index_token
            .setter(msg::sender())
            .delete(order_index);
        self.order_size_delta
            .setter(msg::sender())
            .delete(order_index);
        self.order_is_long.setter(msg::sender()).delete(order_index);
        self.order_trigger_price
            .setter(msg::sender())
            .delete(order_index);
        self.order_trigger_above_threshold
            .setter(msg::sender())
            .delete(order_index);
        self.order_execution_fee
            .setter(msg::sender())
            .delete(order_index);

        safe_transfer(
            self.ctx(),
            collateral_token,
            msg::sender(),
            collateral_amount,
        )?;
        transfer_eth(self, msg::sender(), execution_fee)?;

        evm::log(CancelIncreaseOrder {
            account: msg::sender(),
            order_index,
            collateral_amount,
            collateral_token,
            index_token,
            size_delta,
            is_long,
            trigger_price,
            execution_fee,
            trigger_above_threshold,
        });

        Ok(())
    }

    pub fn get_increase_order(
        &self,
        account: Address,
        order_index: U256,
    ) -> Result<RawIncreaseOrder, Vec<u8>> {
        let existed_account = self.order_account.get(account).get(order_index);
        if existed_account.is_zero() {
            return Err(OrderbookError::OrderNotFound { order_index }.into());
        }

        let collateral_amount = self.order_collateral_amount.get(account).get(order_index);
        let collateral_token = self.order_collateral_token.get(account).get(order_index);
        let index_token = self.order_index_token.get(account).get(order_index);
        let size_delta = self.order_size_delta.get(account).get(order_index);
        let is_long = self.order_is_long.get(account).get(order_index);
        let trigger_price = self.order_trigger_price.get(account).get(order_index);
        let trigger_above_threshold = self
            .order_trigger_above_threshold
            .get(account)
            .get(order_index);
        let execution_fee = self.order_execution_fee.get(account).get(order_index);

        Ok((
            account,
            collateral_amount,
            collateral_token,
            index_token,
            size_delta,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        ))
    }
}
