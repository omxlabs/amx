#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256, U8};
use omx_common::{call_context::GetCallContext, safe_add, safe_sub};
use omx_interfaces::{
    erc20::{safe_transfer, IErc20},
    vault::{
        validate, DecreasePoolAmount, DecreaseReservedAmount, DirectPoolDeposit,
        IncreasePoolAmount, IncreaseReservedAmount, VaultError,
    },
    vault_price_feed::IVaultPriceFeed,
};
use stylus_sdk::{contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct Vault {
        bool initialized;

        address gov;
        address swap_manager;
        address positions_manager;
        address positions_increase_manager;
        address positions_decrease_manager;
        address positions_liquidation_manager;
        address positions_manager_utils;
        address price_feed;

        uint256 whitelisted_token_count;

        uint256 total_token_weights;

        address[] all_whitelisted_tokens;

        mapping (address => bool) whitelisted_tokens;
        mapping (address => uint8) token_decimals;
        mapping (address => uint256) min_profit_basis_points;
        mapping (address => bool) stable_tokens;
        mapping (address => bool) shortable_tokens;

        /// token_balances is used only to determine _transferIn values
        mapping (address => uint256) token_balances;

        /// token_weights allows customisation of index composition
        mapping (address => uint256) token_weights;

        /// pool_amounts tracks the number of received tokens that can be used for leverage
        /// this is tracked separately from tokenBalances to exclude funds that are deposited as margin collateral
        mapping (address => uint256) pool_amounts;

        /// reserved_amounts tracks the number of tokens reserved for open leverage positions
        mapping (address => uint256) reserved_amounts;
    }
}

impl Vault {
    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    /// checks if call is from one of the core vault contracts
    /// such as the swap_manager or positions_manager
    fn only_core(&self) -> Result<(), Vec<u8>> {
        validate(
            msg::sender() == self.swap_manager.get()
                || msg::sender() == self.positions_manager.get()
                || msg::sender() == self.positions_manager_utils.get()
                || msg::sender() == self.positions_liquidation_manager.get()
                || msg::sender() == self.positions_increase_manager.get()
                || msg::sender() == self.positions_decrease_manager.get(),
            VaultError::Forbidden,
        )?;

        Ok(())
    }

    fn transfer_in_internal(&mut self, token: Address) -> Result<U256, Vec<u8>> {
        let prev_balance = self.token_balances.get(token);
        let next_balance = IErc20::new(token).balance_of(self.ctx(), contract::address())?;

        self.token_balances.insert(token, next_balance);

        safe_sub(next_balance, prev_balance)
    }

    fn transfer_out_internal(
        &mut self,
        token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        safe_transfer(self.ctx(), token, receiver, amount)?;

        let balance = IErc20::new(token).balance_of(self.ctx(), contract::address())?;
        self.token_balances.insert(token, balance);

        Ok(())
    }

    fn increase_pool_amount_internal(
        &mut self,
        token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.pool_amounts
            .insert(token, safe_add(self.pool_amounts.get(token), amount)?);

        let balance = IErc20::new(token).balance_of(self.ctx(), contract::address())?;
        validate(
            self.pool_amounts.get(token) <= balance,
            VaultError::PoolExceededBalance,
        )?;

        evm::log(IncreasePoolAmount { amount, token });
        Ok(())
    }
}

#[external]
impl Vault {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        swap_manager: Address,
        positions_manager: Address,
        positions_increase_manager: Address,
        positions_decrease_manager: Address,
        positions_liquidation_manager: Address,
        positions_manager_utils: Address,
        price_feed: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.swap_manager.set(swap_manager);
        self.positions_manager.set(positions_manager);
        self.positions_increase_manager
            .set(positions_increase_manager);
        self.positions_decrease_manager
            .set(positions_decrease_manager);
        self.positions_liquidation_manager
            .set(positions_liquidation_manager);
        self.positions_manager_utils.set(positions_manager_utils);
        self.price_feed.set(price_feed);

        self.initialized.set(true);

        Ok(())
    }

    pub fn is_stable(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(self.stable_tokens.get(token))
    }

    pub fn is_shortable(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(self.shortable_tokens.get(token))
    }

    pub fn is_whitelisted(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(self.whitelisted_tokens.get(token))
    }

    pub fn get_token_decimals(&self, token: Address) -> Result<u8, Vec<u8>> {
        Ok(self.token_decimals.get(token).to())
    }

    pub fn token_weight(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.token_weights.get(token))
    }

    pub fn total_token_weights(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_token_weights.get())
    }

    pub fn reserved_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.reserved_amounts.get(token))
    }

    pub fn min_profit_basis_point(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.min_profit_basis_points.get(token))
    }

    pub fn all_whitelisted_tokens_length(&self) -> Result<U256, Vec<u8>> {
        Ok(U256::from(self.all_whitelisted_tokens.len()))
    }

    pub fn all_whitelisted_tokens(&self, index: u64) -> Result<Address, Vec<u8>> {
        Ok(self.all_whitelisted_tokens.get(index).unwrap_or_default())
    }

    pub fn pool_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.pool_amounts.get(token))
    }

    pub fn transfer_in(&mut self, token: Address) -> Result<U256, Vec<u8>> {
        self.only_core()?;

        self.transfer_in_internal(token)
    }

    pub fn transfer_out(
        &mut self,
        token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        self.only_core()?;

        self.transfer_out_internal(token, amount, receiver)
    }

    pub fn decrease_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.only_core()?;

        self.pool_amounts.insert(
            token,
            safe_sub(self.pool_amounts.get(token), amount).map_err(|_| VaultError::PoolExceeded)?,
        );
        validate(
            self.reserved_amounts.get(token) <= self.pool_amounts.get(token),
            VaultError::PoolLessThenReserved,
        )?;
        evm::log(DecreasePoolAmount { amount, token });
        Ok(())
    }

    pub fn increase_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.only_core()?;

        self.increase_pool_amount_internal(token, amount)
    }

    pub fn update_token_balance(&mut self, token: Address) -> Result<(), Vec<u8>> {
        self.only_core()?;

        let next_balance = IErc20::new(token).balance_of(self.ctx(), contract::address())?;
        self.token_balances.insert(token, next_balance);
        Ok(())
    }

    pub fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        let price = IVaultPriceFeed::new(self.price_feed.get()).get_price(self, token, false)?;

        Ok(price)
    }

    pub fn set_token_config(
        &mut self,
        token: Address,
        token_decimals: u8,
        token_weight: U256,
        min_profit_basis_points: U256,
        is_stable: bool,
        is_shortable: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        // increment token count for the first time
        if !self.whitelisted_tokens.get(token) {
            self.whitelisted_token_count
                .set(safe_add(self.whitelisted_token_count.get(), 1)?);
            self.all_whitelisted_tokens.push(token);
        }

        let total_token_weights = safe_sub(
            self.total_token_weights.get(),
            self.token_weights.get(token),
        )?;

        self.whitelisted_tokens.insert(token, true);
        self.token_decimals.insert(token, U8::from(token_decimals));
        self.token_weights.insert(token, token_weight);
        self.min_profit_basis_points
            .insert(token, min_profit_basis_points);
        self.stable_tokens.insert(token, is_stable);
        self.shortable_tokens.insert(token, is_shortable);

        self.total_token_weights
            .set(safe_add(total_token_weights, token_weight)?);

        // validate price feed
        self.get_price(token)?;

        Ok(())
    }

    pub fn clear_token_config(&mut self, token: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        validate(
            self.whitelisted_tokens.get(token),
            VaultError::TokenNotWhitelisted,
        )?;

        self.total_token_weights.set(safe_sub(
            self.total_token_weights.get(),
            self.token_weights.get(token),
        )?);
        self.whitelisted_tokens.delete(token);
        self.token_decimals.delete(token);
        self.token_weights.delete(token);
        self.min_profit_basis_points.delete(token);
        self.stable_tokens.delete(token);
        self.shortable_tokens.delete(token);
        self.whitelisted_token_count
            .set(safe_sub(self.whitelisted_token_count.get(), 1)?);

        Ok(())
    }

    /// deposit into the pool without minting USDO tokens
    /// useful in allowing the pool to become over-collaterised
    pub fn direct_pool_deposit(&mut self, token: Address) -> Result<(), Vec<u8>> {
        validate(
            self.whitelisted_tokens.get(token),
            VaultError::TokenNotWhitelisted,
        )?;

        let token_amount = self.transfer_in_internal(token)?;
        validate(token_amount > U256::ZERO, VaultError::ZeroAmount)?;

        self.increase_pool_amount_internal(token, token_amount)?;

        evm::log(DirectPoolDeposit {
            token,
            amount: token_amount,
        });

        Ok(())
    }

    pub fn increase_reserved_amount(
        &mut self,
        token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_core()?;

        self.reserved_amounts
            .insert(token, safe_add(self.reserved_amounts.get(token), amount)?);

        validate(
            self.reserved_amounts.get(token) <= self.pool_amounts.get(token),
            VaultError::PoolLessThenReserved,
        )?;

        evm::log(IncreaseReservedAmount { amount, token });
        Ok(())
    }

    pub fn decrease_reserved_amount(
        &mut self,
        token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_core()?;

        self.reserved_amounts
            .insert(token, safe_sub(self.reserved_amounts.get(token), amount)?);
        evm::log(DecreaseReservedAmount { amount, token });
        Ok(())
    }
}
