#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    safe_add, safe_mul, safe_mul_ratio, BASIS_POINTS_DIVISOR, FUNDING_RATE_PRECISION,
};
use omx_interfaces::vault::{validate, IVault, VaultError};
use stylus_sdk::{block, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct VaultUtils {
        bool initialized;

        address vault;
        uint256 min_profit_time;
    }
}
impl VaultUtils {
    fn pool_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).pool_amount(self, token)?)
    }

    fn reserved_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).reserved_amount(self, token)?)
    }

    fn is_stable(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).is_stable(self, token)?)
    }

    fn is_shortable(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).is_shortable(self, token)?)
    }

    fn is_whitelisted(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).is_whitelisted(self, token)?)
    }

    fn token_decimals(&self, token: Address) -> Result<u8, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_token_decimals(self, token)?)
    }

    fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_price(self, token)?)
    }
}

#[external]
impl VaultUtils {
    pub fn init(&mut self, vault: Address, min_profit_time: U256) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.vault.set(vault);
        self.min_profit_time.set(min_profit_time);

        self.initialized.set(true);
        Ok(())
    }

    pub fn get_utilization(&self, token: Address) -> Result<U256, Vec<u8>> {
        let pool_amount = self.pool_amount(token)?;
        if pool_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        safe_mul_ratio(
            self.reserved_amount(token)?,
            FUNDING_RATE_PRECISION,
            pool_amount,
        )
    }

    pub fn validate_tokens(
        &self,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
    ) -> Result<(), Vec<u8>> {
        if is_long {
            validate(
                collateral_token == index_token,
                VaultError::CollateralNotIndex,
            )?;
            validate(
                self.is_whitelisted(collateral_token)?,
                VaultError::CollateralNotWhitelisted,
            )?;
            validate(
                !self.is_stable(collateral_token)?,
                VaultError::CollateralIsStable,
            )?;
            return Ok(());
        }

        validate(
            self.is_whitelisted(collateral_token)?,
            VaultError::CollateralNotWhitelisted,
        )?;
        validate(
            self.is_stable(collateral_token)?,
            VaultError::CollateralNotStable,
        )?;
        validate(!self.is_stable(index_token)?, VaultError::IndexIsStable)?;
        validate(
            self.is_shortable(index_token)?,
            VaultError::IndexNotShortable,
        )?;

        Ok(())
    }

    pub fn token_to_usd(&self, token: Address, token_amount: U256) -> Result<U256, Vec<u8>> {
        if token_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }
        let price = self.get_price(token)?;
        let decimals = self.token_decimals(token)?;

        safe_mul_ratio(
            token_amount,
            price,
            U256::from(10).pow(U256::from(decimals)),
        )
    }

    pub fn usd_to_token(&self, token: Address, usd_amount: U256) -> Result<U256, Vec<u8>> {
        if usd_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let price = self.get_price(token)?;
        let decimals = self.token_decimals(token)?;

        safe_mul_ratio(usd_amount, U256::from(10).pow(U256::from(decimals)), price)
    }

    pub fn get_delta(
        &self,
        index_token: Address,
        size: U256,
        average_price: U256,
        is_long: bool,
        last_increased_time: U256,
    ) -> Result<(bool, U256), Vec<u8>> {
        validate(average_price != U256::ZERO, VaultError::AveragePriceZero)?;

        let price = self.get_price(index_token)?;
        let price_delta: ruint::Uint<256, 4> = average_price.abs_diff(price);
        let mut delta = safe_mul_ratio(size, price_delta, average_price)?;

        let has_profit = if is_long {
            price > average_price
        } else {
            average_price > price
        };

        // if the min_profit_time has passed then there will be no min profit threshold
        // the min profit threshold helps to prevent front-running issues
        let min_bps = if U256::from(block::timestamp())
            > safe_add(last_increased_time, self.min_profit_time.get())?
        {
            U256::ZERO
        } else {
            IVault::new(self.vault.get()).min_profit_basis_point(self, index_token)?
        };

        if has_profit && safe_mul(delta, BASIS_POINTS_DIVISOR)? <= safe_mul(size, min_bps)? {
            delta = U256::ZERO;
        }

        Ok((has_profit, delta))
    }
}
