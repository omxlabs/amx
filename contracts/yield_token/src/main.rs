#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::call_context::GetCallContext;
use omx_interfaces::{
    erc20::{Approval, Transfer},
    yield_token::YieldTokenError,
    yield_tracker::IYieldTracker,
};
use stylus_sdk::{evm, msg, prelude::*};

pub const DECIMALS: u8 = 18;

sol_storage! {
    #[entrypoint]
    pub struct YieldToken {
        bool initialized;

        string name;
        string symbol;

        uint256 total_supply;
        uint256 non_staking_supply;

        address gov;

        mapping (address => bool) is_minter;

        mapping (address => uint256) balances;
        mapping (address => mapping (address => uint256)) allowances;

        address[] yield_trackers;
        mapping (address => bool) non_staking_accounts;
        mapping (address => bool) admins;

        bool in_whitelist_mode;
        mapping (address => bool) whitelisted_handlers;
    }
}

impl YieldToken {
    fn only_gov(&self) -> Result<(), YieldTokenError> {
        if self.gov.get() != msg::sender() {
            return Err(YieldTokenError::Forbidden);
        }

        Ok(())
    }

    fn only_admin(&self) -> Result<(), YieldTokenError> {
        if self.admins.get(msg::sender()) {
            return Err(YieldTokenError::Forbidden);
        }

        Ok(())
    }

    fn only_minter(&self) -> Result<(), YieldTokenError> {
        if !self.is_minter.get(msg::sender()) {
            return Err(YieldTokenError::Forbidden);
        }

        Ok(())
    }

    fn mint_internal(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        if account.is_zero() {
            return Err(YieldTokenError::MintToZeroAddress.into());
        }

        self.update_rewards_internal(account)?;

        self.total_supply.set(self.total_supply.get() + amount);
        self.balances.insert(
            account,
            self.balances
                .get(account)
                .checked_add(amount)
                .ok_or(YieldTokenError::BalanceOverflow)?,
        );

        if self.non_staking_accounts.get(account) {
            self.non_staking_supply.set(
                self.non_staking_supply
                    .get()
                    .checked_add(amount)
                    .ok_or(YieldTokenError::BalanceOverflow)?,
            );
        }

        evm::log(Transfer {
            from: Address::ZERO,
            to: account,
            value: amount,
        });

        Ok(())
    }

    fn burn_internal(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        if account.is_zero() {
            return Err(YieldTokenError::BurnFromZeroAddress.into());
        }

        self.update_rewards_internal(account)?;

        self.balances.insert(
            account,
            self.balances
                .get(account)
                .checked_sub(amount)
                .ok_or(YieldTokenError::BalanceUnderflow)?,
        );
        self.total_supply.set(
            self.total_supply
                .get()
                .checked_sub(amount)
                .ok_or(YieldTokenError::TotalSupplyUnderflow)?,
        );

        if self.non_staking_accounts.get(account) {
            self.non_staking_supply.set(
                self.non_staking_supply
                    .get()
                    .checked_sub(amount)
                    .ok_or(YieldTokenError::NonStakingSupplyUnderflow)?,
            );
        }

        evm::log(Transfer {
            from: account,
            to: Address::ZERO,
            value: amount,
        });

        Ok(())
    }

    fn transfer_internal(
        &mut self,
        sender: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if sender.is_zero() {
            return Err(YieldTokenError::TransferFromZeroAddress.into());
        }

        if recipient.is_zero() {
            return Err(YieldTokenError::TransferToZeroAddress.into());
        }

        if self.in_whitelist_mode.get() && !self.whitelisted_handlers.get(msg::sender()) {
            return Err(YieldTokenError::SenderNotWhitelisted.into());
        }

        self.update_rewards_internal(sender)?;
        self.update_rewards_internal(recipient)?;

        self.balances.insert(
            sender,
            self.balances
                .get(sender)
                .checked_sub(amount)
                .ok_or(YieldTokenError::BalanceUnderflow)?,
        );

        self.balances.insert(
            recipient,
            self.balances
                .get(recipient)
                .checked_add(amount)
                .ok_or(YieldTokenError::BalanceOverflow)?,
        );

        if self.non_staking_accounts.get(sender) {
            self.non_staking_supply.set(
                self.non_staking_supply
                    .get()
                    .checked_sub(amount)
                    .ok_or(YieldTokenError::NonStakingSupplyUnderflow)?,
            );
        }

        if self.non_staking_accounts.get(recipient) {
            self.non_staking_supply.set(
                self.non_staking_supply
                    .get()
                    .checked_add(amount)
                    .ok_or(YieldTokenError::NonStakingSupplyOverflow)?,
            );
        }

        evm::log(Transfer {
            from: sender,
            to: recipient,
            value: amount,
        });

        Ok(())
    }

    fn approve_internal(
        &mut self,
        owner: Address,
        spender: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if owner.is_zero() {
            return Err(YieldTokenError::ApproveFromZeroAddress.into());
        }

        if spender.is_zero() {
            return Err(YieldTokenError::ApproveToZeroAddress.into());
        }

        self.allowances.setter(owner).insert(spender, amount);

        evm::log(Approval {
            owner,
            spender,
            value: amount,
        });

        Ok(())
    }

    fn update_rewards_internal(&mut self, account: Address) -> Result<(), Vec<u8>> {
        for i in 0..self.yield_trackers.len() {
            let yield_tracker = self.yield_trackers.get(i).unwrap();
            IYieldTracker::new(yield_tracker).update_rewards(self.ctx(), account)?;
        }

        Ok(())
    }
}

#[external]
impl YieldToken {
    pub fn init(
        &mut self,
        name: String,
        symbol: String,
        minter: Address,
        initial_supply: U256,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(YieldTokenError::AlreadyInitialized.into());
        }

        self.is_minter.insert(minter, true);
        self.name.set_str(&name);
        self.symbol.set_str(&symbol);

        self.gov.set(msg::sender());
        self.admins.insert(msg::sender(), true);

        if initial_supply != U256::ZERO {
            self.mint_internal(msg::sender(), initial_supply)?;
        }

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_info(&mut self, name: String, symbol: String) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.name.set_str(name);
        self.symbol.set_str(symbol);

        Ok(())
    }

    pub fn set_yield_trackers(&mut self, yield_trackers: Vec<Address>) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        // TODO find a way to do this more efficiently
        self.yield_trackers.erase();
        for v in yield_trackers {
            self.yield_trackers.push(v);
        }

        Ok(())
    }

    pub fn add_admin(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.admins.insert(account, true);

        Ok(())
    }

    pub fn remove_admin(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.admins.insert(account, false);

        Ok(())
    }

    pub fn set_in_whitelist_mode(&mut self, in_whitelist_mode: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.in_whitelist_mode.set(in_whitelist_mode);

        Ok(())
    }

    pub fn set_whitelisted_handler(
        &mut self,
        handler: Address,
        is_whitelisted: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.whitelisted_handlers.insert(handler, is_whitelisted);

        Ok(())
    }

    pub fn add_non_staking_account(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        if !self.non_staking_accounts.get(account) {
            return Err(YieldTokenError::AccountAlreadyMarked.into());
        }

        self.update_rewards_internal(account)?;
        self.non_staking_accounts.insert(account, true);
        self.non_staking_supply.set(
            self.non_staking_supply
                .get()
                .checked_add(self.balances.get(account))
                .ok_or(YieldTokenError::TotalSupplyOverflow)?,
        );

        Ok(())
    }

    pub fn remove_non_staking_account(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        if self.non_staking_accounts.get(account) {
            return Err(YieldTokenError::AccountNotMarked.into());
        }

        self.update_rewards_internal(account)?;
        self.non_staking_accounts.insert(account, false);
        self.non_staking_supply.set(
            self.non_staking_supply
                .get()
                .checked_sub(self.balances.get(account))
                .ok_or(YieldTokenError::TotalSupplyUnderflow)?,
        );

        Ok(())
    }

    pub fn recover_claim(&mut self, account: Address, receiver: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        for i in 0..self.yield_trackers.len() {
            let yield_tracker = self.yield_trackers.get(i).unwrap();
            IYieldTracker::new(yield_tracker).claim(self.ctx(), account, receiver)?;
        }

        Ok(())
    }

    pub fn claim(&mut self, receiver: Address) -> Result<(), Vec<u8>> {
        for i in 0..self.yield_trackers.len() {
            let yield_tracker = self.yield_trackers.get(i).unwrap();
            IYieldTracker::new(yield_tracker).claim(self.ctx(), msg::sender(), receiver)?;
        }

        Ok(())
    }

    pub fn total_staked(&self) -> Result<U256, Vec<u8>> {
        Ok(self
            .total_supply
            .get()
            .checked_sub(self.non_staking_supply.get())
            .ok_or(YieldTokenError::TotalSupplyUnderflow)?)
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.balances.get(account))
    }

    pub fn staked_balance(&self, account: Address) -> Result<U256, Vec<u8>> {
        if self.non_staking_accounts.get(account) {
            return Ok(U256::ZERO);
        }

        Ok(self.balances.get(account))
    }

    pub fn transfer(&mut self, recipient: Address, amount: U256) -> Result<bool, Vec<u8>> {
        self.transfer_internal(msg::sender(), recipient, amount)?;

        Ok(true)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Vec<u8>> {
        Ok(self.allowances.get(owner).get(spender))
    }

    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<bool, Vec<u8>> {
        self.approve_internal(msg::sender(), spender, amount)?;

        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        sender: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<bool, Vec<u8>> {
        let next_allowance = self
            .allowances
            .getter(sender)
            .get(msg::sender())
            .checked_sub(amount)
            .ok_or(YieldTokenError::TransferAmountExceedsAllowance)?;

        self.approve_internal(sender, msg::sender(), next_allowance)?;
        self.transfer_internal(sender, recipient, amount)?;

        Ok(true)
    }

    pub fn set_minter(&mut self, minter: Address, is_active: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_minter.insert(minter, is_active);

        Ok(())
    }

    pub fn mint(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.only_minter()?;

        self.mint_internal(account, amount)?;

        Ok(())
    }

    pub fn burn(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.only_minter()?;

        self.burn_internal(account, amount)?;

        Ok(())
    }
}
