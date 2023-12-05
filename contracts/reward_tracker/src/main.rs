#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{safe_add, safe_sub};
use omx_interfaces::{
    erc20::Transfer, reward_distributor::IRewardDistributor, reward_tracker::RewardTrackerError,
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct RewardTracker {
        bool initialized;

        address gov;

        string name;
        string symbol;

        address reward_tracker_staking;
        address distributor;

        uint256 total_supply;
        mapping (address => uint256) balances;
        mapping (address => mapping (address => uint256)) allowances;

        bool in_private_transfer_mode;
        mapping (address => bool) is_handler;
    }
}

impl RewardTracker {
    fn only_gov(&self) -> Result<(), RewardTrackerError> {
        if self.gov.get() != msg::sender() {
            return Err(RewardTrackerError::Forbidden);
        }

        Ok(())
    }

    fn only_handler(&self) -> Result<(), Vec<u8>> {
        if !self.is_handler.get(msg::sender()) {
            return Err(RewardTrackerError::Forbidden.into());
        }

        Ok(())
    }

    fn transfer_internal(
        &mut self,
        sender: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if sender.is_zero() {
            return Err(RewardTrackerError::TransferFromZeroAddress.into());
        }

        if recipient.is_zero() {
            return Err(RewardTrackerError::TransferToZeroAddress.into());
        }

        if self.in_private_transfer_mode.get() {
            self.only_handler()?;
        }

        self.balances
            .insert(sender, safe_sub(self.balances.get(sender), amount)?);
        self.balances
            .insert(recipient, safe_add(self.balances.get(recipient), amount)?);

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
            return Err(RewardTrackerError::ApproveFromZeroAddress.into());
        }

        if spender.is_zero() {
            return Err(RewardTrackerError::ApproveToZeroAddress.into());
        }

        self.allowances.setter(owner).insert(spender, amount);

        evm::log(Transfer {
            from: owner,
            to: spender,
            value: amount,
        });

        Ok(())
    }
}

#[external]
impl RewardTracker {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        distributor: Address,
        reward_tracker_staking: Address,
        name: String,
        symbol: String,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(RewardTrackerError::AlreadyInitialized.into());
        }

        self.gov.set(gov);

        self.name.set_str(name);
        self.symbol.set_str(symbol);

        self.reward_tracker_staking.set(reward_tracker_staking);
        self.distributor.set(distributor);

        self.initialized.set(true);

        Ok(())
    }

    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_supply.get())
    }

    pub fn is_handler(&self, account: Address) -> Result<bool, Vec<u8>> {
        Ok(self.is_handler.get(account))
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_in_private_transfer_mode(
        &mut self,
        in_private_transfer_mode: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.in_private_transfer_mode.set(in_private_transfer_mode);

        Ok(())
    }

    pub fn set_handler(&mut self, handler: Address, is_active: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_handler.setter(handler).set(is_active);

        Ok(())
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
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
        if self.is_handler.get(msg::sender()) {
            self.transfer_internal(sender, recipient, amount)?;
            return Ok(true);
        }

        let allowance = self.allowances.get(sender).get(msg::sender());
        let next_allowance = safe_sub(allowance, amount).or(Err(
            RewardTrackerError::TransferAmountExceedsAllowance { amount, allowance },
        ))?;

        self.approve_internal(sender, msg::sender(), next_allowance)?;
        self.transfer_internal(sender, recipient, amount)?;

        Ok(true)
    }

    pub fn tokens_per_interval(&self) -> Result<U256, Vec<u8>> {
        Ok(IRewardDistributor::new(self.distributor.get()).tokens_per_interval(self)?)
    }

    pub fn reward_token(&self) -> Result<Address, Vec<u8>> {
        Ok(IRewardDistributor::new(self.distributor.get()).reward_token(self)?)
    }

    pub fn mint_internal(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        if msg::sender() != self.reward_tracker_staking.get() {
            return Err(RewardTrackerError::Forbidden.into());
        }

        if account.is_zero() {
            return Err(RewardTrackerError::MintToZeroAddress.into());
        }

        self.total_supply
            .set(safe_add(self.total_supply.get(), amount)?);
        self.balances
            .insert(account, safe_add(self.balances.get(account), amount)?);

        evm::log(Transfer {
            from: Address::ZERO,
            to: account,
            value: amount,
        });

        Ok(())
    }

    pub fn burn_internal(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        if msg::sender() != self.reward_tracker_staking.get() {
            return Err(RewardTrackerError::Forbidden.into());
        }

        if account.is_zero() {
            return Err(RewardTrackerError::BurnFromZeroAddress.into());
        }

        self.balances
            .insert(account, safe_sub(self.balances.get(account), amount)?);
        self.total_supply
            .set(safe_sub(self.total_supply.get(), amount)?);

        evm::log(Transfer {
            from: account,
            to: Address::ZERO,
            value: amount,
        });

        Ok(())
    }
}
