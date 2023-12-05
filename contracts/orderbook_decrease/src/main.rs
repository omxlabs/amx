#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{call_context::GetCallContext, safe_add, MIN_EXECUTION_FEE};
use omx_interfaces::{
    erc20::{safe_transfer, safe_transfer_from},
    orderbook::{CancelSwapOrder, CreateSwapOrder, OrderbookError, RawSwapOrder},
    weth::IWeth,
};
use stylus_sdk::{contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct OrderbookDecrease {
        bool initialized;
        address gov;
        address weth;
        address swap_router;


        mapping (address => mapping(uint256 =>  address)) order_account;
        mapping (address => mapping(uint256 =>  address)) order_token_in;
        mapping (address => mapping(uint256 =>  address)) order_token_out;
        mapping (address => mapping(uint256 =>  uint256)) order_amount_in;
        mapping (address => mapping(uint256 =>  uint256)) order_min_out;
        mapping (address => mapping(uint256 =>  uint256)) order_trigger_ratio;
        mapping (address => mapping(uint256 =>  bool)) order_trigger_above_threshold;
        mapping (address => mapping(uint256 =>  uint256)) order_execution_fee;

        mapping (address => uint256) orders_index;
    }
}

impl OrderbookDecrease {
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
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        min_out: U256,
        trigger_ratio: U256,
        trigger_above_threshold: bool,
        execution_fee: U256,
    ) -> Result<(), Vec<u8>> {
        let order_index = self.orders_index.get(account);

        self.orders_index
            .insert(account, safe_add(order_index, U256::from(1))?);
        self.order_account
            .setter(account)
            .insert(order_index, account);
        self.order_token_in
            .setter(account)
            .insert(order_index, token_in);
        self.order_token_out
            .setter(account)
            .insert(order_index, token_out);
        self.order_amount_in
            .setter(account)
            .insert(order_index, amount_in);
        self.order_min_out
            .setter(account)
            .insert(order_index, min_out);
        self.order_trigger_ratio
            .setter(account)
            .insert(order_index, trigger_ratio);
        self.order_trigger_above_threshold
            .setter(account)
            .insert(order_index, trigger_above_threshold);
        self.order_execution_fee
            .setter(account)
            .insert(order_index, execution_fee);

        evm::log(CreateSwapOrder {
            account,
            order_index,
            token_in,
            token_out,
            amount_in,
            min_out,
            trigger_ratio,
            trigger_above_threshold,
            execution_fee,
        });

        Ok(())
    }

    fn transfer_in_eth(&mut self) -> Result<(), Vec<u8>> {
        if msg::value() != U256::ZERO {
            let weth = IWeth::new(self.weth.get());
            weth.deposit(self, contract::address())?;
        }

        Ok(())
    }

    fn transfer_out_eth(&mut self, amount_out: U256, receiver: Address) -> Result<(), Vec<u8>> {
        let weth = IWeth::new(self.weth.get());
        weth.withdraw(self, receiver, amount_out)?;

        Ok(())
    }
}

#[external]
impl OrderbookDecrease {
    pub fn init(
        &mut self,
        gov: Address,
        weth: Address,
        swap_router: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(OrderbookError::AlreadyInitialized.into());
        }

        self.gov.set(gov);
        self.weth.set(weth);
        self.swap_router.set(swap_router);

        self.initialized.set(true);

        Ok(())
    }

    #[payable]
    #[allow(clippy::too_many_arguments)]
    pub fn create_swap_order(
        &mut self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        min_out: U256,
        trigger_ratio: U256,
        trigger_above_threshold: bool,
        execution_fee: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        if amount_in == U256::ZERO {
            return Err(OrderbookError::ZeroAmountIn.into());
        }

        if execution_fee < MIN_EXECUTION_FEE {
            return Err(OrderbookError::InsufficientExecutionFee {
                min_execution_fee: MIN_EXECUTION_FEE,
            }
            .into());
        }

        // always need this call because of mandatory execution_fee user has to transfer in ETH
        self.transfer_in_eth()?;

        if msg::value() != execution_fee {
            return Err(OrderbookError::IncorrectFeeTransferred {
                expected: execution_fee,
            }
            .into());
        }
        safe_transfer_from(
            self.ctx(),
            token_in,
            msg::sender(),
            contract::address(),
            amount_in,
        )?;

        self.create_order_internal(
            msg::sender(),
            token_in,
            token_out,
            amount_in,
            min_out,
            trigger_ratio,
            trigger_above_threshold,
            execution_fee,
        )?;

        Ok(())
    }

    pub fn cancel_swap_order(&mut self, order_index: U256) -> Result<(), Vec<u8>> {
        let account = self.order_account.get(msg::sender()).get(order_index);
        if account.is_zero() {
            return Err(OrderbookError::OrderNotFound { order_index }.into());
        }

        let token_in = self.order_token_in.get(msg::sender()).get(order_index);
        let token_out = self.order_token_out.get(msg::sender()).get(order_index);
        let amount_in = self.order_amount_in.get(msg::sender()).get(order_index);
        let min_out = self.order_min_out.get(msg::sender()).get(order_index);
        let trigger_ratio = self.order_trigger_ratio.get(msg::sender()).get(order_index);
        let trigger_above_threshold = self
            .order_trigger_above_threshold
            .get(msg::sender())
            .get(order_index);
        let execution_fee = self.order_execution_fee.get(msg::sender()).get(order_index);

        self.order_account.setter(msg::sender()).delete(order_index);
        self.order_token_in
            .setter(msg::sender())
            .delete(order_index);
        self.order_token_out
            .setter(msg::sender())
            .delete(order_index);
        self.order_amount_in
            .setter(msg::sender())
            .delete(order_index);
        self.order_min_out.setter(msg::sender()).delete(order_index);
        self.order_trigger_ratio
            .setter(msg::sender())
            .delete(order_index);
        self.order_trigger_above_threshold
            .setter(msg::sender())
            .delete(order_index);
        self.order_execution_fee
            .setter(msg::sender())
            .delete(order_index);

        if token_in == self.weth.get() {
            self.transfer_out_eth(safe_add(execution_fee, amount_in)?, msg::sender())?;
        } else {
            safe_transfer(self.ctx(), token_in, msg::sender(), amount_in)?;
            self.transfer_out_eth(execution_fee, msg::sender())?;
        }

        evm::log(CancelSwapOrder {
            account: msg::sender(),
            order_index,
            token_in,
            token_out,
            amount_in,
            execution_fee,
            min_out,
            trigger_ratio,
            trigger_above_threshold,
        });

        Ok(())
    }

    pub fn get_swap_order(
        &self,
        account: Address,
        order_index: U256,
    ) -> Result<RawSwapOrder, Vec<u8>> {
        let existed_account = self.order_account.get(account).get(order_index);
        if existed_account.is_zero() {
            return Err(OrderbookError::OrderNotFound { order_index }.into());
        }

        let token_in = self.order_token_in.get(account).get(order_index);
        let token_out = self.order_token_out.get(account).get(order_index);
        let amount_in = self.order_amount_in.get(account).get(order_index);
        let min_out = self.order_min_out.get(account).get(order_index);
        let trigger_ratio = self.order_trigger_ratio.get(account).get(order_index);
        let trigger_above_threshold = self
            .order_trigger_above_threshold
            .get(account)
            .get(order_index);
        let execution_fee = self.order_execution_fee.get(account).get(order_index);

        Ok((
            account,
            token_in,
            token_out,
            amount_in,
            min_out,
            trigger_ratio,
            trigger_above_threshold,
            execution_fee,
        ))
    }
}
