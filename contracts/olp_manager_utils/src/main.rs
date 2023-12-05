#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    safe_add, safe_div, safe_mul, safe_mul_ratio, safe_sub, BASIS_POINTS_DIVISOR,
    OLP_MANAGER_AUM_ADDITION, OLP_MANAGER_AUM_DEDUCTION, OLP_PRECISION, PRICE_PRECISION,
    USDO_DECIMALS,
};
use omx_interfaces::{
    base_token::IBaseToken,
    olp_manager::OlpManagerError,
    vault::{IPositionsManager, IShortsTracker, IVault},
};
use stylus_sdk::{msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct OlpManagerUtils {
        bool initialized;

        address gov;

        address vault;
        address positions_manager;
        address olp;
        address shorts_tracker;

        uint256 shorts_tracker_average_price_weight;
    }
}

impl OlpManagerUtils {
    fn only_gov(&self) -> Result<(), OlpManagerError> {
        if self.gov.get() != msg::sender() {
            return Err(OlpManagerError::Forbidden);
        }

        Ok(())
    }
}

#[external]
impl OlpManagerUtils {
    pub fn init(
        &mut self,
        vault: Address,
        positions_manager: Address,
        shorts_tracker: Address,
        olp: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(OlpManagerError::AlreadyInitialized.into());
        }

        self.vault.set(vault);
        self.positions_manager.set(positions_manager);
        self.shorts_tracker.set(shorts_tracker);
        self.olp.set(olp);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_shorts_tracker_average_price_weight(&mut self, weight: U256) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        if weight > BASIS_POINTS_DIVISOR {
            return Err(OlpManagerError::InvalidWeight {
                max_weight: BASIS_POINTS_DIVISOR,
            }
            .into());
        }

        self.shorts_tracker_average_price_weight.set(weight);

        Ok(())
    }

    pub fn get_price(&self) -> Result<U256, Vec<u8>> {
        let aum = self.get_aum()?;
        let olp = IBaseToken::new(self.olp.get());
        let supply = olp.total_supply(self)?;

        safe_mul_ratio(aum, OLP_PRECISION, supply)
    }

    pub fn get_aum_in_usdo(&self) -> Result<U256, Vec<u8>> {
        let aum = self.get_aum()?;
        let scale = U256::from(10).pow(U256::from(USDO_DECIMALS));

        safe_mul_ratio(aum, scale, PRICE_PRECISION)
    }

    pub fn get_aum(&self) -> Result<U256, Vec<u8>> {
        let vault = IVault::new(self.vault.get());

        let length: u64 = vault.all_whitelisted_tokens_length(self)?.to();

        let mut aum = OLP_MANAGER_AUM_ADDITION;
        let mut short_profits = U256::ZERO;

        for i in 0..length {
            let token = vault.all_whitelisted_tokens(self, i)?;
            let is_whitelisted = vault.is_whitelisted(self, token)?;

            if !is_whitelisted {
                continue;
            }

            let price = vault.get_price(self, token)?;
            let pool_amount = vault.pool_amount(self, token)?;
            let decimals = vault.get_token_decimals(self, token)?;
            let token_scale = U256::from(10).pow(U256::from(decimals));

            if vault.is_stable(self, token)? {
                aum = safe_add(aum, safe_mul_ratio(pool_amount, price, token_scale)?)?;
            } else {
                let positions_manager = IPositionsManager::new(self.positions_manager.get());
                // add global short profit / loss
                let size = positions_manager.global_short_size(self, token)?;

                if size > U256::ZERO {
                    let (delta, has_profit) = self.get_global_short_delta(token, price, size)?;
                    if !has_profit {
                        // add losses from shorts
                        aum = safe_add(aum, delta)?;
                    } else {
                        short_profits = safe_add(short_profits, delta)?;
                    }
                }

                aum = safe_add(aum, positions_manager.guaranteed_usd(self, token)?)?;

                let reserved_amount = vault.reserved_amount(self, token)?;
                aum = safe_add(
                    aum,
                    safe_mul_ratio(safe_sub(pool_amount, reserved_amount)?, price, token_scale)?,
                )?;
            }
        }

        let aum = aum.checked_sub(short_profits).unwrap_or(U256::ZERO);

        Ok(aum
            .checked_sub(OLP_MANAGER_AUM_DEDUCTION)
            .unwrap_or(U256::ZERO))
    }

    pub fn get_global_short_delta(
        &self,
        token: Address,
        price: U256,
        size: U256,
    ) -> Result<(U256, bool), Vec<u8>> {
        let average_price = self.get_global_short_average_price(token)?;
        let price_delta = if average_price > price {
            safe_sub(average_price, price)?
        } else {
            safe_sub(price, average_price)?
        };
        let delta = safe_mul_ratio(size, price_delta, average_price)?;
        Ok((delta, average_price > price))
    }

    pub fn get_global_short_average_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        let shorts_tracker = IShortsTracker::new(self.shorts_tracker.get());
        if self.shorts_tracker.get().is_zero() || shorts_tracker.is_global_short_data_ready(self)? {
            return self.get_global_short_average_price(token);
        }

        let shorts_tracker_average_price_weight = self.shorts_tracker_average_price_weight.get();
        if shorts_tracker_average_price_weight == U256::ZERO {
            return self.get_global_short_average_price(token);
        } else if shorts_tracker_average_price_weight == PRICE_PRECISION {
            return Ok(shorts_tracker.global_short_average_prices(self, token)?);
        }

        let vault_average_price = self.get_global_short_average_price(token)?;
        let shorts_tracker_average_price =
            shorts_tracker.global_short_average_prices(self, token)?;

        safe_div(
            safe_add(
                safe_mul(
                    vault_average_price,
                    safe_sub(BASIS_POINTS_DIVISOR, shorts_tracker_average_price_weight)?,
                )?,
                safe_mul(
                    shorts_tracker_average_price,
                    shorts_tracker_average_price_weight,
                )?,
            )?,
            BASIS_POINTS_DIVISOR,
        )
    }
}
