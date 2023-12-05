#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    safe_add, safe_div, safe_mul, safe_mul_ratio, safe_sub, FUNDING_INTERVAL, FUNDING_RATE_FACTOR,
    STABLE_FUNDING_RATE_FACTOR,
};
use omx_interfaces::vault::{validate, IVault, UpdateFundingRate, VaultError};
use stylus_sdk::{block, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct FundingRateManager {
        bool initialized;

        address gov;
        address vault;

        /// cumulative_funding_rates tracks the funding rates based on utilization
        mapping (address => uint256) cumulative_funding_rates;
        /// last_funding_times tracks the last time funding was updated for a token
        mapping (address => uint256) last_funding_times;
    }
}

impl FundingRateManager {
    fn only_initialized(&self) -> Result<(), Vec<u8>> {
        validate(self.initialized.get(), VaultError::NotInitialized)?;

        Ok(())
    }

    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    fn pool_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).pool_amount(self, token)?)
    }

    fn is_stable(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).is_stable(self, token)?)
    }

    fn reserved_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).reserved_amount(self, token)?)
    }
}

#[external]
impl FundingRateManager {
    pub fn init(&mut self, gov: Address, vault: Address) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.vault.set(vault);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn cumulative_funding_rate(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.cumulative_funding_rates.get(token))
    }

    pub fn get_next_funding_rate(&self, token: Address) -> Result<U256, Vec<u8>> {
        let time = U256::from(block::timestamp());
        if safe_add(self.last_funding_times.get(token), FUNDING_INTERVAL)? > time {
            return Ok(U256::ZERO);
        }

        let intervals = safe_div(
            safe_sub(time, self.last_funding_times.get(token))?,
            FUNDING_INTERVAL,
        )?;
        let pool_amount = self.pool_amount(token)?;
        if pool_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let funding_rate_factor = if self.is_stable(token)? {
            STABLE_FUNDING_RATE_FACTOR
        } else {
            FUNDING_RATE_FACTOR
        };

        safe_mul_ratio(
            safe_mul(funding_rate_factor, self.reserved_amount(token)?)?,
            intervals,
            pool_amount,
        )
    }

    pub fn update_cumulative_funding_rate(
        &mut self,
        collateral_token: Address,
    ) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        let time = U256::from(block::timestamp());
        if self.last_funding_times.get(collateral_token) == U256::ZERO {
            self.last_funding_times
                .insert(collateral_token, time / FUNDING_INTERVAL * FUNDING_INTERVAL);
            return Ok(());
        }

        if safe_add(
            self.last_funding_times.get(collateral_token),
            FUNDING_INTERVAL,
        )? > time
        {
            return Ok(());
        }

        let funding_rate = self.get_next_funding_rate(collateral_token)?;
        self.cumulative_funding_rates.insert(
            collateral_token,
            safe_add(
                self.cumulative_funding_rates.get(collateral_token),
                funding_rate,
            )?,
        );
        self.last_funding_times
            .insert(collateral_token, time / FUNDING_INTERVAL * FUNDING_INTERVAL);

        evm::log(UpdateFundingRate {
            token: collateral_token,
            funding_rate: self.cumulative_funding_rates.get(collateral_token),
        });

        Ok(())
    }
}
