#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, I256, U256};
use omx_common::{safe_add, safe_mul_ratio, safe_sub};
use omx_interfaces::{
    shorts_tracker::{GlobalShortDataUpdated, ShortsTrackerError},
    vault::{IPositionsManager, IVault, IVaultUtils, RawPositionData},
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct ShortsTracker {
        bool initialized;

        address gov;

        address vault;
        address vault_utils;
        address positions_manager;


        mapping (address => bool) is_handler;
        mapping (bytes32 => bytes32) data;

        mapping (address => uint256) global_short_average_prices;
        bool is_global_short_data_ready;
    }
}

impl ShortsTracker {
    fn only_gov(&self) -> Result<(), ShortsTrackerError> {
        if self.gov.get() != msg::sender() {
            return Err(ShortsTrackerError::Forbidden);
        }

        Ok(())
    }

    fn only_handler(&self) -> Result<(), ShortsTrackerError> {
        if !self.is_handler.get(msg::sender()) {
            return Err(ShortsTrackerError::Forbidden);
        }

        Ok(())
    }

    pub fn get_next_global_average_price(
        &self,
        average_price: U256,
        next_price: U256,
        next_size: U256,
        delta: U256,
        realised_pnl: I256,
    ) -> Result<U256, Vec<u8>> {
        let (has_profit, next_delta) =
            self.get_next_delta(average_price, next_price, delta, realised_pnl)?;

        let size = if has_profit {
            safe_sub(next_size, next_delta)?
        } else {
            safe_add(next_size, next_delta)?
        };
        let next_average_price = safe_mul_ratio(next_price, next_size, size)?;

        Ok(next_average_price)
    }

    fn get_next_delta(
        &self,
        average_price: U256,
        next_price: U256,
        mut delta: U256,
        realised_pnl: I256,
    ) -> Result<(bool, U256), Vec<u8>> {
        // global delta 10000, realised pnl 1000 => new pnl 9000
        // global delta 10000, realised pnl -1000 => new pnl 11000
        // global delta -10000, realised pnl 1000 => new pnl -11000
        // global delta -10000, realised pnl -1000 => new pnl -9000
        // global delta 10000, realised pnl 11000 => new pnl -1000 (flips sign)
        // global delta -10000, realised pnl -11000 => new pnl 1000 (flips sign)
        let mut has_profit = average_price > next_price;
        if has_profit {
            if realised_pnl > I256::ZERO {
                let realised_pnl = realised_pnl.unsigned_abs();

                if realised_pnl > delta {
                    delta = safe_sub(realised_pnl, delta)?;
                    has_profit = false;
                } else {
                    delta = safe_sub(delta, realised_pnl)?;
                }
            } else {
                delta = safe_add(delta, realised_pnl.unsigned_abs())?;
            }

            return Ok((has_profit, delta));
        }

        if realised_pnl > I256::ZERO {
            delta = safe_add(delta, realised_pnl.unsigned_abs())?;
        } else {
            let realised_pnl = realised_pnl.unsigned_abs();
            if realised_pnl > delta {
                delta = safe_sub(realised_pnl, delta)?;
                has_profit = true;
            } else {
                delta = safe_sub(delta, realised_pnl)?;
            }
        }

        Ok((has_profit, delta))
    }
}

#[external]
impl ShortsTracker {
    pub fn init(
        &mut self,
        gov: Address,
        vault: Address,
        vault_utils: Address,
        positions_manager: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(ShortsTrackerError::AlreadyInitialized.into());
        }

        self.gov.set(gov);

        self.vault.set(vault);
        self.vault_utils.set(vault_utils);
        self.positions_manager.set(positions_manager);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_handler(&mut self, handler: Address, is_active: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_handler.insert(handler, is_active);

        Ok(())
    }

    pub fn set_is_global_short_data_ready(&mut self, value: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_global_short_data_ready.set(value);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_global_short_data(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
        size_delta: U256,
        mark_price: U256,
        is_increase: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_handler()?;

        if is_long || size_delta == U256::ZERO {
            return Ok(());
        }

        if !self.is_global_short_data_ready.get() {
            return Ok(());
        }

        let (global_short_size, global_short_average_price) = self.get_next_global_short_data(
            account,
            collateral_token,
            index_token,
            mark_price,
            size_delta,
            is_increase,
        )?;
        self.global_short_average_prices
            .insert(index_token, global_short_average_price);

        evm::log(GlobalShortDataUpdated {
            token: index_token,
            global_short_size,
            global_short_average_price,
        });

        Ok(())
    }

    pub fn get_global_short_delta(&self, token: Address) -> Result<(bool, U256), Vec<u8>> {
        let positions_manager = IPositionsManager::new(self.positions_manager.get());
        let size = positions_manager.global_short_size(self, token)?;

        let average_price = self.global_short_average_prices.get(token);
        if size == U256::ZERO {
            return Ok((false, U256::ZERO));
        }

        let vault = IVault::new(self.vault.get());

        let next_price = vault.get_price(self, token)?;
        let price_delta = next_price.abs_diff(average_price);
        let delta = safe_mul_ratio(size, price_delta, average_price)?;

        Ok((average_price > next_price, delta))
    }

    pub fn set_init_data(
        &mut self,
        tokens: Vec<Address>,
        average_prices: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        if self.is_global_short_data_ready.get() {
            return Err(ShortsTrackerError::AlreadyMigrated.into());
        }

        for (token, average_price) in tokens.into_iter().zip(average_prices.into_iter()) {
            self.global_short_average_prices
                .insert(token, average_price);
        }

        self.is_global_short_data_ready.set(true);

        Ok(())
    }

    pub fn get_next_global_short_data(
        &self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        next_price: U256,
        size_delta: U256,
        is_increase: bool,
    ) -> Result<(U256, U256), Vec<u8>> {
        let realised_pnl = self.get_realised_pnl(
            account,
            collateral_token,
            index_token,
            size_delta,
            is_increase,
        )?;
        let average_price = self.global_short_average_prices.get(index_token);
        let price_delta = average_price.abs_diff(next_price);

        let positions_manager = IPositionsManager::new(self.positions_manager.get());
        let size = positions_manager.global_short_size(self, index_token)?;
        let next_size = if is_increase {
            safe_add(size, size_delta)?
        } else {
            safe_sub(size, size_delta)?
        };

        if next_size == U256::ZERO {
            return Ok((U256::ZERO, U256::ZERO));
        }

        if average_price == U256::ZERO {
            return Ok((next_size, next_price));
        }

        let delta = safe_mul_ratio(size, price_delta, average_price)?;

        let next_average_price = self.get_next_global_average_price(
            average_price,
            next_price,
            next_size,
            delta,
            realised_pnl,
        )?;

        Ok((next_size, next_average_price))
    }

    pub fn get_realised_pnl(
        &self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        size_delta: U256,
        is_increase: bool,
    ) -> Result<I256, Vec<u8>> {
        if is_increase {
            return Ok(I256::ZERO);
        }

        let positions_manager = IPositionsManager::new(self.positions_manager.get());
        let vault_utils = IVaultUtils::new(self.vault_utils.get());

        let (size, _, average_price, .., last_increased_time): RawPositionData =
            positions_manager.position(self, account, collateral_token, index_token, false)?;

        let (has_profit, delta) = vault_utils.get_delta(
            self,
            index_token,
            size,
            average_price,
            false,
            last_increased_time,
        )?;
        let adjusted_delta = safe_mul_ratio(size_delta, delta, size)?;
        if adjusted_delta > I256::MAX.unsigned_abs() {
            return Err(ShortsTrackerError::Overflow.into());
        }

        let adjusted_delta = adjusted_delta.try_into().unwrap();
        Ok(if has_profit {
            adjusted_delta
        } else {
            -adjusted_delta
        })
    }
}
