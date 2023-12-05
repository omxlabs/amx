#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, I256, U256};
use omx_common::{
    safe_add, safe_mul, safe_mul_ratio, safe_sub, BASIS_POINTS_DIVISOR, LIQUIDATION_FEE_USD,
    MAX_LEVERAGE,
};
use omx_interfaces::vault::{
    position::Position, validate, IFeeManager, IPositionsManager, IVault, IVaultUtils, UpdatePnl,
    VaultError,
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsManagerUtils {
        bool initialized;

        address positions_manager;
        address positions_decrease_manager;
        address vault;
        address fee_manager;
        address vault_utils;
    }
}

impl PositionsManagerUtils {
    fn only_manager(&self) -> Result<(), Vec<u8>> {
        validate(
            msg::sender() == self.positions_decrease_manager.get(),
            VaultError::Forbidden,
        )?;

        Ok(())
    }

    fn position(
        &self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
    ) -> Result<Position, Vec<u8>> {
        Ok(IPositionsManager::new(self.positions_manager.get())
            .position(self, account, collateral_token, index_token, is_long)?
            .into())
    }

    fn decrease_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_pool_amount(self, token, amount)?;

        Ok(())
    }

    fn collect_margin_fees(
        &mut self,
        collateral_token: Address,
        size_delta: U256,
        size: U256,
        entry_funding_rate: U256,
    ) -> Result<U256, Vec<u8>> {
        Ok(
            IFeeManager::new(self.fee_manager.get()).collect_margin_fees(
                self,
                collateral_token,
                size_delta,
                size,
                entry_funding_rate,
            )?,
        )
    }

    fn get_delta(
        &self,
        index_token: Address,
        size: U256,
        average_price: U256,
        is_long: bool,
        last_increased_time: U256,
    ) -> Result<(bool, U256), Vec<u8>> {
        Ok(IVaultUtils::new(self.vault_utils.get()).get_delta(
            self,
            index_token,
            size,
            average_price,
            is_long,
            last_increased_time,
        )?)
    }

    fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_price(self, token)?)
    }

    fn token_decimals(&self, token: Address) -> Result<u8, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_token_decimals(self, token)?)
    }

    fn usd_to_token(&self, token: Address, usd_amount: U256) -> Result<U256, Vec<u8>> {
        if usd_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let price = self.get_price(token)?;
        let decimals = self.token_decimals(token)?;

        safe_mul_ratio(usd_amount, U256::from(10).pow(U256::from(decimals)), price)
    }

    fn increase_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).increase_pool_amount(self, token, amount)?;

        Ok(())
    }
}

#[external]
impl PositionsManagerUtils {
    pub fn init(
        &mut self,
        positions_manager: Address,
        positions_decrease_manager: Address,
        vault: Address,
        fee_manager: Address,
        vault_utils: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.positions_manager.set(positions_manager);
        self.positions_decrease_manager
            .set(positions_decrease_manager);
        self.vault.set(vault);
        self.fee_manager.set(fee_manager);
        self.vault_utils.set(vault_utils);

        self.initialized.set(true);

        Ok(())
    }

    pub fn reduce_collateral(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        collateral_delta: U256,
        size_delta: U256,
        is_long: bool,
    ) -> Result<(U256, U256), Vec<u8>> {
        self.only_manager()?;

        let mut position = self.position(account, collateral_token, index_token, is_long)?;

        let fee = self.collect_margin_fees(
            collateral_token,
            size_delta,
            position.size,
            position.entry_funding_rate,
        )?;

        let (has_profit, delta) = self.get_delta(
            index_token,
            position.size,
            position.average_price,
            is_long,
            position.last_increased_time,
        )?;
        let adjusted_delta = safe_mul_ratio(size_delta, delta, position.size)?;

        let initial_position = position;

        // transfer profits out
        let mut usd_out = U256::ZERO;
        if has_profit && adjusted_delta > U256::ZERO {
            usd_out = adjusted_delta;

            position.realised_pnl = position
                .realised_pnl
                .checked_add(I256::try_from(adjusted_delta).map_err(|_| VaultError::PnlOverflow)?)
                .ok_or(VaultError::PnlOverflow)?;

            // pay out realised profits from the pool amount for short positions
            if !is_long {
                let token_amount = self.usd_to_token(collateral_token, adjusted_delta)?;
                self.decrease_pool_amount(collateral_token, token_amount)?;
            }
        }

        if !has_profit && adjusted_delta > U256::ZERO {
            position.collateral = safe_sub(position.collateral, adjusted_delta)?;
            position.realised_pnl = position
                .realised_pnl
                .checked_sub(I256::try_from(adjusted_delta).map_err(|_| VaultError::PnlOverflow)?)
                .ok_or(VaultError::PnlOverflow)?;

            // transfer realised losses to the pool for short positions
            // realised losses for long positions are not transferred here as
            // [`increase_pool_amount`] was already called in increase_position for longs
            if !is_long {
                let token_amount = self.usd_to_token(collateral_token, adjusted_delta)?;
                self.increase_pool_amount(collateral_token, token_amount)?;
            }
        }

        // reduce the position's collateral by collateral_delta
        // transfer collateral_delta out
        if collateral_delta > U256::ZERO {
            usd_out = safe_add(usd_out, collateral_delta)?;
            position.collateral = safe_sub(position.collateral, collateral_delta)?;
        }

        // if the position will be closed, then transfer the remaining collateral out
        if position.size == size_delta {
            usd_out = safe_add(usd_out, position.collateral)?;
            position.collateral = U256::ZERO;
        }

        // if the usdOut is more than the fee then deduct the fee from the usdOut directly
        // else deduct the fee from the position's collateral
        let usd_out_after_fee = if usd_out > fee {
            safe_sub(usd_out, fee)?
        } else {
            position.collateral = safe_sub(position.collateral, fee)?;

            if is_long {
                let fee_tokens = self.usd_to_token(collateral_token, fee)?;
                self.decrease_pool_amount(collateral_token, fee_tokens)?;
            }

            usd_out
        };

        let positions_manager = IPositionsManager::new(self.positions_manager.get());
        if position != initial_position {
            positions_manager.position_update(
                self,
                account,
                collateral_token,
                index_token,
                is_long,
                position.size,
                position.collateral,
                position.average_price,
                position.entry_funding_rate,
                position.reserve_amount,
                position.realised_pnl,
                position.last_increased_time,
            )?;
        }

        evm::log(UpdatePnl {
            account,
            collateral_token,
            index_token,
            is_long,
            has_profit,
            delta: adjusted_delta,
        });

        Ok((usd_out, usd_out_after_fee))
    }

    pub fn validate_liquidation(
        &self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
        raise: bool,
    ) -> Result<(U256, U256), Vec<u8>> {
        let position = self.position(account, collateral_token, index_token, is_long)?;

        let (has_profit, delta) = self.get_delta(
            index_token,
            position.size,
            position.average_price,
            is_long,
            position.last_increased_time,
        )?;

        let fee_manager = IFeeManager::new(self.fee_manager.get());
        let margin_fees = fee_manager.get_funding_fee(
            self,
            collateral_token,
            position.size,
            position.entry_funding_rate,
        )?;
        let margin_fees = safe_add(
            margin_fees,
            fee_manager.get_position_fee(self, position.size)?,
        )?;

        if !has_profit && position.collateral < delta {
            if raise {
                return Err(VaultError::LossesExceedCollateral.into());
            }
            return Ok((U256::from(1), margin_fees));
        }

        let remaining_collateral = if has_profit {
            position.collateral
        } else {
            safe_sub(position.collateral, delta)?
        };

        if remaining_collateral < margin_fees {
            if raise {
                return Err(VaultError::FeesExceedCollateral.into());
            }
            return Ok((U256::from(1), remaining_collateral));
        }

        if remaining_collateral < safe_add(margin_fees, LIQUIDATION_FEE_USD)? {
            if raise {
                return Err(VaultError::LiquidationFeesExceedCollateral.into());
            }
            return Ok((U256::from(1), margin_fees));
        }

        if safe_mul(remaining_collateral, MAX_LEVERAGE)?
            < safe_mul(position.size, BASIS_POINTS_DIVISOR)?
        {
            if raise {
                return Err(VaultError::MaxLeverageExceeded.into());
            }
            return Ok((U256::from(2), margin_fees));
        }

        Ok((U256::from(1), margin_fees))
    }

    /// for longs: next_average_price = (next_price * next_size)/ (next_size + delta)
    /// for shorts: next_average_price = (next_price * next_size) / (next_size - delta)
    #[allow(clippy::too_many_arguments)]
    pub fn get_next_average_price(
        &self,
        index_token: Address,
        size: U256,
        average_price: U256,
        is_long: bool,
        next_price: U256,
        size_delta: U256,
        last_increased_time: U256,
    ) -> Result<U256, Vec<u8>> {
        let (has_profit, delta) = self.get_delta(
            index_token,
            size,
            average_price,
            is_long,
            last_increased_time,
        )?;
        let next_size = safe_add(size, size_delta)?;
        let divisor = if is_long == has_profit {
            safe_sub(next_size, delta)?
        } else {
            safe_add(next_size, delta)?
        };

        safe_mul_ratio(next_price, next_size, divisor)
    }
}
