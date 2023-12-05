#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    call_context::GetCallContext, safe_add, safe_div, safe_mul, safe_mul_ratio, safe_sub,
    PRICE_PRECISION,
};
use omx_interfaces::{
    distributor::IDistributor,
    erc20::{safe_transfer, IErc20},
    yield_token::IYieldToken,
    yield_tracker::{Claim, YieldTrackerError},
};
use stylus_sdk::{contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct YieldTracker {
        bool initialized;

        address gov;
        address yield_token;
        address distributor;

        uint256 cumulative_reward_per_token;
        mapping (address => uint256) claimable_reward;
        mapping (address => uint256) previous_cumulated_reward_per_token;
    }
}

impl YieldTracker {
    pub fn only_gov(&self) -> Result<(), YieldTrackerError> {
        if self.gov.get() != msg::sender() {
            return Err(YieldTrackerError::Forbidden);
        }

        Ok(())
    }
}

#[external]
impl YieldTracker {
    pub fn init(&mut self, gov: Address, yield_token: Address) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(YieldTrackerError::AlreadyInitialized.into());
        }

        self.gov.set(gov);
        self.yield_token.set(yield_token);
        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_distributor(&mut self, distributor: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.distributor.set(distributor);

        Ok(())
    }

    pub fn claim(&mut self, account: Address, receiver: Address) -> Result<U256, Vec<u8>> {
        if msg::sender() != self.yield_token.get() {
            return Err(YieldTrackerError::Forbidden.into());
        }

        self.update_rewards(account)?;

        let token_amount = self.claimable_reward.get(account);
        self.claimable_reward.insert(account, U256::ZERO);

        let distributor = IDistributor::new(self.distributor.get());
        let reward_token = distributor.get_reward_token(self.ctx(), contract::address())?;

        safe_transfer(self.ctx(), reward_token, receiver, token_amount)?;
        evm::log(Claim {
            amount: token_amount,
            receiver: account,
        });

        Ok(token_amount)
    }

    pub fn get_tokens_per_interval(&self) -> Result<U256, Vec<u8>> {
        todo!("YieldTracker: get_tokens_per_interval");
    }

    pub fn claimable(&self, account: Address) -> Result<U256, Vec<u8>> {
        let staked_balance = IErc20::new(self.yield_token.get()).balance_of(self, account)?;
        if staked_balance == U256::ZERO {
            return Ok(self.claimable_reward.get(account));
        }

        let distribution_amount = IDistributor::new(self.distributor.get())
            .get_distribution_amount(self, contract::address())?;
        let pending_rewards = safe_mul(distribution_amount, PRICE_PRECISION)?;

        let total_staked = IYieldToken::new(self.yield_token.get()).total_staked(self)?;

        let next_cumulative_reward_per_token = safe_add(
            self.cumulative_reward_per_token.get(),
            safe_div(pending_rewards, total_staked)?,
        )?;

        safe_mul_ratio(
            staked_balance,
            safe_sub(
                next_cumulative_reward_per_token,
                self.previous_cumulated_reward_per_token.get(account),
            )?,
            PRICE_PRECISION,
        )
    }

    pub fn update_rewards(&mut self, account: Address) -> Result<(), Vec<u8>> {
        let mut block_reward = U256::ZERO;

        if self.distributor.get() != Address::ZERO {
            block_reward = IDistributor::new(self.distributor.get()).distribute(self.ctx())?;
        }

        let total_staked = IYieldToken::new(self.yield_token.get()).total_staked(self.ctx())?;

        // only update cumulative_reward_per_token when there are stakers, i.e. when total_staked > 0
        // if block_reward == 0, then there will be no change to cumulative_reward_per_token
        if total_staked > U256::ZERO && block_reward > U256::ZERO {
            let new_cumulative_reward = safe_add(
                self.cumulative_reward_per_token.get(),
                safe_div(safe_mul(block_reward, PRICE_PRECISION)?, total_staked)?,
            )?;

            self.cumulative_reward_per_token.set(new_cumulative_reward);
        }

        // cumulative_reward_per_token can only increase
        // so if cumulative_reward_per_token is zero, it means there are no rewards yet
        if self.cumulative_reward_per_token.get() == U256::ZERO {
            return Ok(());
        }

        if account != Address::ZERO {
            let staked_balance =
                IYieldToken::new(self.yield_token.get()).staked_balance(self.ctx(), account)?;

            let cumulative_reward_delta = safe_sub(
                self.cumulative_reward_per_token.get(),
                self.previous_cumulated_reward_per_token.get(account),
            )?;

            let claimable_reward = safe_add(
                self.claimable_reward.get(account),
                safe_mul_ratio(staked_balance, cumulative_reward_delta, PRICE_PRECISION)?,
            )?;

            self.claimable_reward.insert(account, claimable_reward);
            self.previous_cumulated_reward_per_token
                .insert(account, self.cumulative_reward_per_token.get());
        }

        Ok(())
    }
}
