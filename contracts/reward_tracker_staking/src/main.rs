#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    call_context::GetCallContext, safe_add, safe_mul_ratio, safe_sub, PRICE_PRECISION as PRECISION,
};
use omx_interfaces::{
    erc20::{safe_transfer, safe_transfer_from},
    reward_distributor::IRewardDistributor,
    reward_tracker::{Claim, IRewardTracker, RewardTrackerError},
};
use stylus_sdk::{contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct RewardTrackerStaking {
        bool initialized;

        address gov;

        address distributor;
        address reward_tracker;

        uint256 cumulative_reward_per_token;

        mapping (address => mapping (address => uint256)) deposit_balances;
        mapping (address => uint256) total_deposit_supply;

        mapping (address => bool) is_deposit_token;
        mapping (address => uint256) claimable_reward;
        mapping (address => uint256) staked_amounts;
        mapping (address => uint256) average_staked_amounts;
        mapping (address => uint256) previous_cumulated_reward_per_token;
        mapping (address => uint256) cumulative_rewards;

        bool in_private_staking_mode;
        bool in_private_claiming_mode;
    }
}

impl RewardTrackerStaking {
    fn only_gov(&self) -> Result<(), RewardTrackerError> {
        if self.gov.get() != msg::sender() {
            return Err(RewardTrackerError::Forbidden);
        }

        Ok(())
    }

    fn only_handler(&self) -> Result<(), Vec<u8>> {
        let is_handler =
            IRewardTracker::new(self.reward_tracker.get()).is_handler(self, msg::sender())?;

        if is_handler {
            Ok(())
        } else {
            Err(RewardTrackerError::Forbidden.into())
        }
    }

    pub fn reward_token(&self) -> Result<Address, Vec<u8>> {
        Ok(IRewardDistributor::new(self.distributor.get()).reward_token(self)?)
    }

    fn claim_internal(&mut self, account: Address, receiver: Address) -> Result<U256, Vec<u8>> {
        self.update_rewards_internal(account)?;

        let token_amount = self.claimable_reward.get(account);
        self.claimable_reward.insert(account, U256::ZERO);

        if token_amount > U256::ZERO {
            let reward_token = self.reward_token()?;
            safe_transfer(self.ctx(), reward_token, receiver, token_amount)?;
            evm::log(Claim {
                receiver,
                amount: token_amount,
            });
        }

        Ok(token_amount)
    }

    fn stake_internal(
        &mut self,
        funding_account: Address,
        account: Address,
        deposit_token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if amount == U256::ZERO {
            return Err(RewardTrackerError::InvalidZeroAmount.into());
        }

        if !self.is_deposit_token.get(deposit_token) {
            return Err(RewardTrackerError::InvalidDepositToken.into());
        }

        safe_transfer_from(
            self.ctx(),
            deposit_token,
            funding_account,
            contract::address(),
            amount,
        )?;

        self.update_rewards_internal(account)?;

        self.staked_amounts
            .insert(account, safe_add(self.staked_amounts.get(account), amount)?);
        let new_deposit_balance = safe_add(
            self.deposit_balances.get(account).get(deposit_token),
            amount,
        )?;
        self.deposit_balances
            .setter(account)
            .insert(deposit_token, new_deposit_balance);
        self.total_deposit_supply.insert(
            deposit_token,
            safe_add(self.total_deposit_supply.get(deposit_token), amount)?,
        );

        IRewardTracker::new(self.reward_tracker.get()).mint_internal(self, account, amount)?;

        Ok(())
    }

    fn unstake_internal(
        &mut self,
        account: Address,
        deposit_token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        if amount == U256::ZERO {
            return Err(RewardTrackerError::InvalidZeroAmount.into());
        }

        if !self.is_deposit_token.get(deposit_token) {
            return Err(RewardTrackerError::InvalidDepositToken.into());
        }

        self.update_rewards_internal(account)?;

        let staked_amount = self.staked_amounts.get(account);
        if staked_amount < amount {
            return Err(RewardTrackerError::AmountExceedsStakedAmount {
                staked_amount,
                amount,
            }
            .into());
        }

        self.staked_amounts
            .insert(account, safe_sub(staked_amount, amount)?);

        let deposit_balance = self.deposit_balances.get(account).get(deposit_token);

        let new_deposit_balance = safe_sub(deposit_balance, amount).or(Err(
            RewardTrackerError::AmountExceedsDepositBalance {
                deposit_balance,
                amount,
            },
        ))?;
        self.deposit_balances
            .setter(account)
            .insert(deposit_token, new_deposit_balance);
        self.total_deposit_supply.insert(
            deposit_token,
            safe_sub(self.total_deposit_supply.get(deposit_token), amount)?,
        );

        let reward_tracker = IRewardTracker::new(self.reward_tracker.get());
        reward_tracker.burn_internal(self.ctx(), account, amount)?;
        safe_transfer(self, deposit_token, receiver, amount)?;

        Ok(())
    }

    fn update_rewards_internal(&mut self, account: Address) -> Result<(), Vec<u8>> {
        let distributor = IRewardDistributor::new(self.distributor.get());
        let block_reward = distributor.distribute(self.ctx())?;

        let reward_tracker = IRewardTracker::new(self.reward_tracker.get());
        let supply = reward_tracker.total_supply(self.ctx())?;

        if supply > U256::ZERO && block_reward > U256::ZERO {
            self.cumulative_reward_per_token.set(safe_add(
                self.cumulative_reward_per_token.get(),
                safe_mul_ratio(block_reward, PRECISION, supply)?,
            )?);
        }

        // cumulative_reward_per_token can only increase
        // so if cumulative_reward_per_token is zero, it means there are no rewards yet
        if self.cumulative_reward_per_token.get() == U256::ZERO {
            return Ok(());
        }

        if !account.is_zero() {
            let staked_amount = self.staked_amounts.get(account);
            let account_reward = safe_mul_ratio(
                staked_amount,
                safe_sub(
                    self.cumulative_reward_per_token.get(),
                    self.previous_cumulated_reward_per_token.get(account),
                )?,
                PRECISION,
            )?;
            let claimable_reward = safe_add(self.claimable_reward.get(account), account_reward)?;

            self.claimable_reward.insert(account, claimable_reward);
            self.previous_cumulated_reward_per_token
                .insert(account, self.cumulative_reward_per_token.get());

            if claimable_reward > U256::ZERO && self.staked_amounts.get(account) > U256::ZERO {
                let next_cumulative_reward =
                    safe_add(self.cumulative_rewards.get(account), account_reward)?;

                self.average_staked_amounts.insert(
                    account,
                    safe_add(
                        safe_mul_ratio(
                            self.average_staked_amounts.get(account),
                            self.cumulative_rewards.get(account),
                            next_cumulative_reward,
                        )?,
                        safe_mul_ratio(staked_amount, account_reward, next_cumulative_reward)?,
                    )?,
                );

                self.cumulative_rewards
                    .insert(account, next_cumulative_reward);
            }
        }

        Ok(())
    }
}

#[external]
impl RewardTrackerStaking {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        reward_tracker: Address,
        distributor: Address,
        deposit_tokens: Vec<Address>,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(RewardTrackerError::AlreadyInitialized.into());
        }

        self.gov.set(gov);

        for deposit_token in deposit_tokens.into_iter() {
            self.is_deposit_token.insert(deposit_token, true);
        }

        self.distributor.set(distributor);
        self.reward_tracker.set(reward_tracker);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_deposit_token(
        &mut self,
        deposit_token: Address,
        is_deposit_token: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_deposit_token
            .setter(deposit_token)
            .set(is_deposit_token);

        Ok(())
    }

    pub fn deposit_balance(
        &self,
        account: Address,
        deposit_token: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.deposit_balances.get(account).get(deposit_token))
    }

    pub fn stake(&mut self, deposit_token: Address, amount: U256) -> Result<(), Vec<u8>> {
        if self.in_private_staking_mode.get() {
            return Err(RewardTrackerError::ActionNotEnabled.into());
        }

        self.stake_internal(msg::sender(), msg::sender(), deposit_token, amount)?;

        Ok(())
    }

    pub fn set_in_private_claiming_mode(
        &mut self,
        in_private_claiming_mode: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.in_private_claiming_mode.set(in_private_claiming_mode);

        Ok(())
    }

    pub fn set_in_private_staking_mode(
        &mut self,
        in_private_staking_mode: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.in_private_staking_mode.set(in_private_staking_mode);

        Ok(())
    }

    pub fn stake_for_account(
        &mut self,
        funding_account: Address,
        account: Address,
        deposit_token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_handler()?;

        self.stake_internal(funding_account, account, deposit_token, amount)?;

        Ok(())
    }

    pub fn unstake(&mut self, deposit_token: Address, amount: U256) -> Result<(), Vec<u8>> {
        if self.in_private_staking_mode.get() {
            return Err(RewardTrackerError::ActionNotEnabled.into());
        }

        self.unstake_internal(msg::sender(), deposit_token, amount, msg::sender())?;

        Ok(())
    }

    pub fn unstake_for_account(
        &mut self,
        account: Address,
        deposit_token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        self.only_handler()?;

        self.unstake_internal(account, deposit_token, amount, receiver)?;

        Ok(())
    }

    pub fn update_rewards(&mut self) -> Result<(), Vec<u8>> {
        self.update_rewards_internal(Address::ZERO)?;

        Ok(())
    }

    pub fn claim(&mut self, receiver: Address) -> Result<U256, Vec<u8>> {
        if self.in_private_claiming_mode.get() {
            return Err(RewardTrackerError::ActionNotEnabled.into());
        }

        self.claim_internal(msg::sender(), receiver)
    }

    pub fn claim_for_account(
        &mut self,
        account: Address,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        self.only_handler()?;

        self.claim_internal(account, receiver)
    }

    pub fn claimable(&self, account: Address) -> Result<U256, Vec<u8>> {
        let staked_amount = self.staked_amounts.get(account);
        if staked_amount == U256::ZERO {
            return Ok(self.claimable_reward.get(account));
        }

        let supply = IRewardTracker::new(self.reward_tracker.get()).total_supply(self)?;
        let pending_rewards =
            IRewardDistributor::new(self.distributor.get()).pending_rewards(self)?;
        let next_cumulative_reward_per_token = safe_add(
            self.cumulative_reward_per_token.get(),
            safe_mul_ratio(pending_rewards, PRECISION, supply)?,
        )?;

        safe_add(
            self.claimable_reward.get(account),
            safe_mul_ratio(
                staked_amount,
                safe_sub(
                    next_cumulative_reward_per_token,
                    self.previous_cumulated_reward_per_token.get(account),
                )?,
                PRECISION,
            )?,
        )
    }

    pub fn staked_amount(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.staked_amounts.get(account))
    }

    pub fn cumulative_reward(&self, account: Address) -> Result<U256, Vec<u8>> {
        Ok(self.cumulative_rewards.get(account))
    }
}
