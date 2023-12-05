#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, I256, U256};
use omx_common::{safe_add, safe_add_int, safe_mul_ratio, safe_sub};
use omx_interfaces::vault::{
    get_position_key, position::RawPositionData, validate, DecreaseGuaranteedUsd,
    IncreaseGuaranteedUsd, VaultError,
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsManager {
        bool initialized;

        address gov;
        address vault_utils;
        address fee_manager;
        address positions_decrease_manager;
        address positions_increase_manager;
        address positions_liquidation_manager;
        address positions_manager_utils;

        mapping (address => uint256) global_short_sizes;
        mapping (address => uint256) global_short_average_prices;
        mapping (address => uint256) max_global_short_sizes;

        mapping (bytes32 => uint256) position_size;
        mapping (bytes32 => uint256) position_collateral;
        mapping (bytes32 => uint256) position_average_price;
        mapping (bytes32 => uint256) position_entry_funding_rate;
        mapping (bytes32 => uint256) position_reserve_amount;
        mapping (bytes32 => uint256) position_last_increased_time;
        mapping (bytes32 => int256) position_realised_pnl;

        /// guaranteed_usd tracks the amount of USD that is "guaranteed" by opened leverage positions
        /// this value is used to calculate the redemption values for selling of USDO
        /// this is an estimated amount, it is possible for the actual guaranteed value to be lower
        /// in the case of sudden price decreases, the guaranteed value should be corrected
        /// after liquidations are carried out
        mapping (address => uint256) guaranteed_usd;
    }
}

impl PositionsManager {
    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    fn only_manager(&self) -> Result<(), Vec<u8>> {
        validate(
            self.positions_decrease_manager.get() == msg::sender()
                || self.positions_increase_manager.get() == msg::sender()
                || self.positions_liquidation_manager.get() == msg::sender()
                || self.positions_manager_utils.get() == msg::sender(),
            VaultError::Forbidden,
        )?;

        Ok(())
    }

    fn increase_global_short_size(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        let global_short_sizes = safe_add(self.global_short_sizes.get(token), amount)?;

        let max_size = self.max_global_short_sizes.get(token);
        if max_size != U256::ZERO {
            validate(
                global_short_sizes <= max_size,
                VaultError::MaxShortsExceeded,
            )?;
        }

        self.global_short_sizes.insert(token, global_short_sizes);

        Ok(())
    }
}

#[external]
impl PositionsManager {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        vault_utils: Address,
        fee_manager: Address,
        positions_decrease_manager: Address,
        positions_increase_manager: Address,
        positions_liquidation_manager: Address,
        positions_manager_utils: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.vault_utils.set(vault_utils);
        self.fee_manager.set(fee_manager);
        self.positions_decrease_manager
            .set(positions_decrease_manager);
        self.positions_increase_manager
            .set(positions_increase_manager);
        self.positions_liquidation_manager
            .set(positions_liquidation_manager);
        self.positions_manager_utils.set(positions_manager_utils);

        self.initialized.set(true);

        Ok(())
    }

    pub fn update_guaranteed_usd(&mut self, token: Address, value: I256) -> Result<(), Vec<u8>> {
        self.only_manager()?;

        let guaranteed_usd = safe_add_int(self.guaranteed_usd.get(token), value)?;

        self.guaranteed_usd.insert(token, guaranteed_usd);

        if value.is_positive() {
            evm::log(IncreaseGuaranteedUsd {
                amount: value.unsigned_abs(),
                token,
            });
        } else {
            evm::log(DecreaseGuaranteedUsd {
                amount: value.unsigned_abs(),
                token,
            });
        }

        Ok(())
    }

    pub fn guaranteed_usd(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.guaranteed_usd.get(token))
    }

    pub fn after_short_increase(
        &mut self,
        index_token: Address,
        price: U256,
        size_delta: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_manager()?;

        if self.global_short_sizes.get(index_token) == U256::ZERO {
            self.global_short_average_prices
                .setter(index_token)
                .set(price);
        } else {
            let new_average_price =
                self.get_next_global_short_average_price(index_token, price, size_delta)?;
            self.global_short_average_prices
                .setter(index_token)
                .set(new_average_price);
        }

        self.increase_global_short_size(index_token, size_delta)?;

        Ok(())
    }

    pub fn decrease_global_short_size(
        &mut self,
        token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_manager()?;

        let size = self.global_short_sizes.get(token);
        let amount = if amount > size {
            U256::ZERO
        } else {
            size - amount
        };

        self.global_short_sizes.insert(token, amount);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn position(
        &self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
    ) -> Result<RawPositionData, Vec<u8>> {
        let key = get_position_key(account, collateral_token, index_token, is_long);

        Ok((
            self.position_size.get(key),
            self.position_collateral.get(key),
            self.position_average_price.get(key),
            self.position_entry_funding_rate.get(key),
            self.position_reserve_amount.get(key),
            self.position_realised_pnl.get(key),
            self.position_last_increased_time.get(key),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn position_update(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
        size: U256,
        collateral: U256,
        average_price: U256,
        entry_funding_rate: U256,
        reserve_amount: U256,
        realised_pnl: I256,
        last_increased_time: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_manager()?;

        let key = get_position_key(account, collateral_token, index_token, is_long);

        self.position_size.insert(key, size);
        self.position_collateral.insert(key, collateral);
        self.position_average_price.insert(key, average_price);
        self.position_entry_funding_rate
            .insert(key, entry_funding_rate);
        self.position_reserve_amount.insert(key, reserve_amount);
        self.position_realised_pnl.insert(key, realised_pnl);
        self.position_last_increased_time
            .insert(key, last_increased_time);

        Ok(())
    }

    /// for longs: next_average_price = (next_price * next_size)/ (next_size + delta)
    /// for shorts: next_average_price = (next_price * next_size) / (next_size - delta)
    pub fn get_next_global_short_average_price(
        &self,
        index_token: Address,
        next_price: U256,
        size_delta: U256,
    ) -> Result<U256, Vec<u8>> {
        let size = self.global_short_sizes.get(index_token);
        let average_price = self.global_short_average_prices.get(index_token);
        let price_delta = average_price.abs_diff(next_price);
        let delta = safe_mul_ratio(size, price_delta, average_price)?;
        let has_profit = average_price > next_price;

        let next_size = safe_add(size, size_delta)?;
        let divisor = if has_profit {
            safe_sub(next_size, delta)?
        } else {
            safe_add(next_size, delta)?
        };

        safe_mul_ratio(next_price, next_size, divisor)
    }

    pub fn global_short_size(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.global_short_sizes.get(token))
    }
}
