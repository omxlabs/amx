#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    call_context::GetCallContext, safe_div, safe_mul, safe_sub, DISTRIBUTION_INTERVAL,
};
use omx_interfaces::{
    distributor::{DistributionChange, DistributorError, TokensPerIntervalChange},
    erc20::{safe_transfer, IErc20},
};
use stylus_sdk::{block, contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct TimeDistributor {
        bool initialized;

        address gov;
        address admin;

        mapping (address => address) reward_tokens;
        mapping (address => uint256) tokens_per_interval;
        mapping (address => uint256) last_distribution_time;
    }
}

impl TimeDistributor {
    fn only_gov(&self) -> Result<(), DistributorError> {
        if self.gov.get() != msg::sender() {
            return Err(DistributorError::Forbidden);
        }

        Ok(())
    }

    fn only_admin(&self) -> Result<(), DistributorError> {
        if self.admin.get() != msg::sender() {
            return Err(DistributorError::Forbidden);
        }

        Ok(())
    }

    fn update_last_distribution_time_internal(&mut self, receiver: Address) -> Result<(), Vec<u8>> {
        let now = U256::from(block::timestamp());

        self.last_distribution_time.insert(
            receiver,
            now / DISTRIBUTION_INTERVAL * DISTRIBUTION_INTERVAL,
        );

        Ok(())
    }
}

#[external]
impl TimeDistributor {
    pub fn init(&mut self) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(DistributorError::AlreadyInitialized.into());
        }

        self.admin.set(msg::sender());
        self.gov.set(msg::sender());

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_tokens_per_interval(
        &mut self,
        receiver: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        if self.last_distribution_time.get(receiver) != U256::ZERO {
            let intervals = self.get_intervals(receiver)?;
            if intervals != U256::ZERO {
                return Err(DistributorError::PendingDistribution.into());
            }
        }

        self.tokens_per_interval.insert(receiver, amount);
        self.update_last_distribution_time_internal(receiver)?;

        evm::log(TokensPerIntervalChange { receiver, amount });

        Ok(())
    }

    pub fn update_last_distribution_time(&mut self, receiver: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;

        self.update_last_distribution_time_internal(receiver)?;

        Ok(())
    }

    pub fn set_distribution(
        &mut self,
        receivers: Vec<Address>,
        amounts: Vec<U256>,
        reward_tokens: Vec<Address>,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        for (receiver, amount, reward_token) in itertools::izip!(receivers, amounts, reward_tokens)
        {
            if self.last_distribution_time.get(receiver) != U256::ZERO {
                let intervals = self.get_intervals(receiver)?;
                if intervals != U256::ZERO {
                    return Err(DistributorError::PendingDistribution.into());
                }
            }

            self.tokens_per_interval.insert(receiver, amount);
            self.reward_tokens.insert(receiver, reward_token);
            self.update_last_distribution_time_internal(receiver)?;

            evm::log(DistributionChange {
                receiver,
                amount,
                reward_token,
            });
        }

        Ok(())
    }

    pub fn distribute(&mut self) -> Result<U256, Vec<u8>> {
        let receiver = msg::sender();
        let intervals = self.get_intervals(receiver)?;

        if intervals == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let amount = self.get_distribution_amount(receiver)?;
        self.update_last_distribution_time_internal(receiver)?;

        if amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let reward_token = self.reward_tokens.get(receiver);
        safe_transfer(self.ctx(), reward_token, receiver, amount)?;

        evm::log(omx_interfaces::distributor::Distribute { receiver, amount });

        Ok(amount)
    }

    pub fn get_reward_token(&self, receiver: Address) -> Result<Address, Vec<u8>> {
        Ok(self.reward_tokens.get(receiver))
    }

    pub fn get_distribution_amount(&self, receiver: Address) -> Result<U256, Vec<u8>> {
        let tokens_per_interval = self.tokens_per_interval.get(receiver);
        if tokens_per_interval == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let intervals = self.get_intervals(receiver)?;
        let amount = safe_mul(tokens_per_interval, intervals)?;

        let reward_token = IErc20::new(self.reward_tokens.get(receiver));
        if reward_token.balance_of(self, contract::address())? < amount {
            Ok(U256::ZERO)
        } else {
            Ok(amount)
        }
    }

    pub fn get_intervals(&self, receiver: Address) -> Result<U256, Vec<u8>> {
        let now = U256::from(block::timestamp());
        let last_distribution_time = self.last_distribution_time.get(receiver);

        let time_diff = safe_sub(now, last_distribution_time)?;

        safe_div(time_diff, DISTRIBUTION_INTERVAL)
    }
}
