#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    call_context::GetCallContext, safe_mul_ratio, safe_sub, BASIS_POINTS_DIVISOR, BONUS_DURATION,
};
use omx_interfaces::{
    bonus_distributor::{BonusDistributorError, BonusMultiplierChange, Distribute},
    erc20::{safe_transfer, IErc20},
    reward_tracker::IRewardTrackerStaking,
};
use stylus_sdk::{block, contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct BonusDistributor {
        bool initialized;

        address gov;

        uint256 bonus_multiplier_basis_points;

        address reward_token;
        address reward_tracker_staking;
        uint256 last_distribution_time;
        address reward_tracker;

        address admin;

    }
}

impl BonusDistributor {
    fn only_gov(&self) -> Result<(), BonusDistributorError> {
        if self.gov.get() != msg::sender() {
            return Err(BonusDistributorError::Forbidden);
        }

        Ok(())
    }

    fn only_admin(&self) -> Result<(), BonusDistributorError> {
        if self.admin.get() != msg::sender() {
            return Err(BonusDistributorError::Forbidden);
        }

        Ok(())
    }
}

#[external]
impl BonusDistributor {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        reward_token: Address,
        reward_tracker: Address,
        reward_tracker_staking: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(BonusDistributorError::AlreadyInitialized.into());
        }

        self.gov.set(gov);
        self.admin.set(gov);

        self.reward_token.set(reward_token);
        self.reward_tracker.set(reward_tracker);
        self.reward_tracker_staking.set(reward_tracker_staking);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_admin(&mut self, admin: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.admin.set(admin);

        Ok(())
    }

    pub fn update_last_distribution_time(&mut self) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        self.last_distribution_time
            .set(U256::from(block::timestamp()));

        Ok(())
    }

    pub fn set_bonus_multiplier(
        &mut self,
        bonus_multiplier_basis_points: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        if self.last_distribution_time.get() == U256::ZERO {
            return Err(BonusDistributorError::ZeroLastDistributionTime.into());
        }

        let reward_tracker_staking = IRewardTrackerStaking::new(self.reward_tracker_staking.get());
        reward_tracker_staking.update_rewards(self.ctx())?;

        self.bonus_multiplier_basis_points
            .set(bonus_multiplier_basis_points);

        evm::log(BonusMultiplierChange {
            amount: bonus_multiplier_basis_points,
        });

        Ok(())
    }

    pub fn tokens_per_interval(&self) -> Result<U256, Vec<u8>> {
        let supply = IErc20::new(self.reward_tracker.get()).total_supply(self)?;
        let bonus_multiplier_basis_points = self.bonus_multiplier_basis_points.get();
        let bonus_duration = BONUS_DURATION;

        Ok(
            safe_mul_ratio(supply, bonus_multiplier_basis_points, BASIS_POINTS_DIVISOR)?
                / bonus_duration,
        )
    }

    pub fn pending_rewards(&self) -> Result<U256, Vec<u8>> {
        let now = U256::from(block::timestamp());
        if now == self.last_distribution_time.get() {
            return Ok(U256::ZERO);
        }

        let supply = IErc20::new(self.reward_tracker.get()).total_supply(self)?;
        let time_diff = safe_sub(now, self.last_distribution_time.get())?;

        safe_mul_ratio(
            safe_mul_ratio(time_diff, supply, BASIS_POINTS_DIVISOR)?,
            self.bonus_multiplier_basis_points.get(),
            BONUS_DURATION,
        )
    }

    pub fn distribute(&mut self) -> Result<U256, Vec<u8>> {
        if msg::sender() != self.reward_tracker.get()
            && msg::sender() != self.reward_tracker_staking.get()
        {
            return Err(BonusDistributorError::InvalidSender {}.into());
        }

        let amount = self.pending_rewards()?;
        if amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        self.last_distribution_time
            .set(U256::from(block::timestamp()));

        let reward_token = IErc20::new(self.reward_token.get());

        let balance = reward_token.balance_of(self.ctx(), contract::address())?;

        let amount = amount.min(balance);

        safe_transfer(self, reward_token, msg::sender(), amount)?;

        evm::log(Distribute { amount });

        Ok(amount)
    }

    pub fn reward_token(&self) -> Result<Address, Vec<u8>> {
        Ok(self.reward_token.get())
    }
}
