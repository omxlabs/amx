#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{safe_add, safe_sub, ETH_DECIMALS};
use omx_interfaces::{
    erc20::{Approval, Transfer},
    weth::WethError,
};
use stylus_sdk::{call::transfer_eth, console, evm, msg, prelude::*};

pub const DECIMALS: u8 = 18;

sol_storage! {
    #[entrypoint]
    pub struct Weth {
        bool initialized;

        string name;
        string symbol;

        uint256 total_supply;

        mapping (address => uint256) balances;
        mapping (address => mapping (address => uint256)) allowances;
    }
}

impl Weth {
    pub fn transfer_internal(
        &mut self,
        sender: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if sender.is_zero() {
            return Err(WethError::TransferFromZeroAddress.into());
        }

        if recipient.is_zero() {
            return Err(WethError::TransferToZeroAddress.into());
        }

        let new_sender_balance = safe_sub(self.balances.get(sender), amount)
            .map_err(|_| WethError::InsufficientBalance)?;
        self.balances.insert(sender, new_sender_balance);

        let new_recipient_balance = safe_add(self.balances.get(recipient), amount)?;
        self.balances.insert(recipient, new_recipient_balance);

        evm::log(Transfer {
            from: sender,
            to: recipient,
            value: amount,
        });

        Ok(())
    }

    pub fn mint_internal(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        if account.is_zero() {
            return Err(WethError::MintToZeroAddress.into());
        }

        self.total_supply
            .set(safe_add(self.total_supply.get(), amount)?);

        let new_account_balance = safe_add(self.balances.get(account), amount)?;
        self.balances.insert(account, new_account_balance);

        evm::log(Transfer {
            from: Address::ZERO,
            to: account,
            value: amount,
        });

        Ok(())
    }

    pub fn burn_internal(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        if account.is_zero() {
            return Err(WethError::BurnFromZeroAddress.into());
        }

        let new_account_balance = safe_sub(self.balances.get(account), amount)
            .map_err(|_| WethError::InsufficientBalance)?;
        self.balances.insert(account, new_account_balance);

        self.total_supply
            .set(safe_sub(self.total_supply.get(), amount)?);

        evm::log(Transfer {
            from: account,
            to: Address::ZERO,
            value: amount,
        });

        Ok(())
    }

    pub fn approve_internal(
        &mut self,
        owner: Address,
        spender: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if owner.is_zero() {
            return Err(WethError::ApproveFromZeroAddress.into());
        }

        if spender.is_zero() {
            return Err(WethError::ApproveToZeroAddress.into());
        }

        self.allowances.setter(owner).insert(spender, amount);

        evm::log(Approval {
            owner,
            spender,
            value: amount,
        });

        Ok(())
    }
}

#[external]
impl Weth {
    pub fn init(&mut self, name: String, symbol: String) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(WethError::AlreadyInitialized.into());
        }

        self.name.set_str(&name);
        self.symbol.set_str(&symbol);

        self.initialized.set(true);

        Ok(())
    }

    #[payable]
    pub fn deposit(&mut self, to: Address) -> Result<(), Vec<u8>> {
        self.balances
            .insert(to, safe_add(self.balances.get(to), msg::value())?);

        Ok(())
    }

    #[payable]
    pub fn deposit_approve(&mut self, to: Address) -> Result<(), Vec<u8>> {
        let amount = msg::value();
        self.balances
            .insert(msg::sender(), safe_add(self.balances.get(to), amount)?);

        self.approve(to, amount)?;

        Ok(())
    }

    pub fn withdraw(&mut self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let sender_balance = self.balances.get(msg::sender());
        let new_spender_balance =
            safe_sub(sender_balance, amount).map_err(|_| WethError::InsufficientBalance)?;

        self.balances.insert(msg::sender(), new_spender_balance);

        transfer_eth(self, to, amount)?;

        Ok(())
    }

    pub fn name(&self) -> Result<String, Vec<u8>> {
        let bytes = self.name.0.get_bytes();
        Ok(String::from_utf8_lossy(&bytes).into())
    }

    pub fn symbol(&self) -> Result<String, Vec<u8>> {
        let bytes = self.symbol.0.get_bytes();
        Ok(String::from_utf8_lossy(&bytes).into())
    }

    pub fn decimals(&self) -> Result<u8, Vec<u8>> {
        Ok(ETH_DECIMALS)
    }

    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_supply.get())
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.balances.get(account))
    }

    pub fn transfer(&mut self, recipient: Address, amount: U256) -> Result<bool, Vec<u8>> {
        self.transfer_internal(msg::sender(), recipient, amount)?;

        Ok(true)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Vec<u8>> {
        Ok(self.allowances.getter(owner).get(spender))
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
        self.transfer_internal(sender, recipient, amount)?;

        let new_sender_allowance =
            safe_sub(self.allowances.getter(sender).get(msg::sender()), amount)
                .map_err(|_| WethError::InsufficientAllowance)?;
        self.allowances
            .setter(sender)
            .insert(msg::sender(), new_sender_allowance);

        Ok(true)
    }

    pub fn increase_allowance(
        &mut self,
        spender: Address,
        added_value: U256,
    ) -> Result<bool, Vec<u8>> {
        let new_spender_allowance = safe_add(
            self.allowances.getter(msg::sender()).get(spender),
            added_value,
        )?;
        self.allowances
            .setter(msg::sender())
            .insert(spender, new_spender_allowance);

        Ok(true)
    }

    pub fn decrease_allowance(
        &mut self,
        spender: Address,
        subtracted_value: U256,
    ) -> Result<bool, Vec<u8>> {
        let new_spender_allowance = safe_sub(
            self.allowances.getter(msg::sender()).get(spender),
            subtracted_value,
        )
        .map_err(|_| WethError::AllowanceBelowZero)?;
        self.allowances
            .setter(msg::sender())
            .insert(spender, new_spender_allowance);

        Ok(true)
    }
}
