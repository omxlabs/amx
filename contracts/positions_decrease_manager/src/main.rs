#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, I256, U256};
use omx_common::{call_context::GetCallContext, safe_mul_ratio, safe_sub, safe_sub_to_int};
use omx_interfaces::vault::{
    position::{validate_position, Position},
    validate, ClosePosition, DecreasePosition, IFundingRateManager, IPositionsManager,
    IPositionsManagerUtils, IVault, UpdatePosition, VaultError,
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsDecreaseManager {
        bool initialized;
        address gov;
        address vault;
        address funding_rate_manager;
        address decrease_router;
        address positions_manager;
        address positions_liquidation_manager;
        address positions_manager_utils;
    }
}

impl PositionsDecreaseManager {
    fn only_initialized(&self) -> Result<(), Vec<u8>> {
        validate(self.initialized.get(), VaultError::NotInitialized)?;

        Ok(())
    }

    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    fn only_liquidation_manager(&self) -> Result<(), Vec<u8>> {
        validate(
            msg::sender() == self.positions_liquidation_manager.get(),
            VaultError::Forbidden,
        )?;

        Ok(())
    }

    fn validate_router(&self, account: Address) -> Result<(), Vec<u8>> {
        if msg::sender() == account || msg::sender() == self.decrease_router.get() {
            return Ok(());
        }

        Ok(())
    }

    fn update_guaranteed_usd(&mut self, token: Address, value: I256) -> Result<(), Vec<u8>> {
        Ok(IPositionsManager::new(self.positions_manager.get())
            .update_guaranteed_usd(self, token, value)?)
    }

    fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_price(self, token)?)
    }

    fn token_decimals(&self, token: Address) -> Result<u8, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_token_decimals(self, token)?)
    }

    fn cumulative_funding_rate(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IFundingRateManager::new(self.funding_rate_manager.get())
            .cumulative_funding_rate(self, token)?)
    }

    fn usd_to_token(&self, token: Address, usd_amount: U256) -> Result<U256, Vec<u8>> {
        if usd_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let price = self.get_price(token)?;
        let decimals = self.token_decimals(token)?;

        safe_mul_ratio(usd_amount, U256::from(10).pow(U256::from(decimals)), price)
    }

    fn decrease_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_pool_amount(self, token, amount)?;

        Ok(())
    }

    fn decrease_reserved_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_reserved_amount(self, token, amount)?;

        Ok(())
    }

    fn position_update(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
        position: Position,
    ) -> Result<(), Vec<u8>> {
        IPositionsManager::new(self.positions_manager.get()).position_update(
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

        Ok(())
    }

    fn transfer_out(
        &mut self,
        token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        Ok(IVault::new(self.vault.get()).transfer_out(self, token, amount, receiver)?)
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
}

#[external]
impl PositionsDecreaseManager {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        vault: Address,
        funding_rate_manager: Address,
        decrease_router: Address,
        positions_manager: Address,
        positions_liquidation_manager: Address,
        positions_manager_utils: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.vault.set(vault);
        self.funding_rate_manager.set(funding_rate_manager);
        self.decrease_router.set(decrease_router);
        self.positions_manager.set(positions_manager);
        self.positions_liquidation_manager
            .set(positions_liquidation_manager);
        self.positions_manager_utils.set(positions_manager_utils);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn decrease_position(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        collateral_delta: U256,
        size_delta: U256,
        is_long: bool,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        self.only_initialized()?;

        self.validate_router(account)
            .or_else(|_| self.only_liquidation_manager())?;

        let positions_manager = IPositionsManager::new(self.positions_manager.get());

        let (reserve_amount, reserve_delta, collateral) = {
            let position = self.position(account, collateral_token, index_token, is_long)?;
            validate(position.size > U256::ZERO, VaultError::ZeroSize)?;
            validate(position.size >= size_delta, VaultError::SizeLessThenDelta)?;
            validate(
                position.collateral >= collateral_delta,
                VaultError::CollateralLessThenDelta,
            )?;

            let collateral = position.collateral;
            let reserve_delta = safe_mul_ratio(position.reserve_amount, size_delta, position.size)?;

            let reserve_amount = safe_sub(position.reserve_amount, reserve_delta)?;

            (reserve_amount, reserve_delta, collateral)
        };

        let mut position = self.position(account, collateral_token, index_token, is_long)?;
        position.reserve_amount = reserve_amount;

        self.position_update(account, collateral_token, index_token, is_long, position)?;

        self.decrease_reserved_amount(collateral_token, reserve_delta)?;

        let positions_manager_utils =
            IPositionsManagerUtils::new(self.positions_manager_utils.get());
        let (usd_out, usd_out_after_fee) = positions_manager_utils.reduce_collateral(
            self.ctx(),
            account,
            collateral_token,
            index_token,
            collateral_delta,
            size_delta,
            is_long,
        )?;

        let mut position = self.position(account, collateral_token, index_token, is_long)?;

        if position.size != size_delta {
            position.entry_funding_rate = self.cumulative_funding_rate(collateral_token)?;
            position.size = safe_sub(position.size, size_delta)?;

            self.position_update(account, collateral_token, index_token, is_long, position)?;

            let collateral = {
                let position = self.position(account, collateral_token, index_token, is_long)?;
                let collateral = position.collateral;
                validate_position(position.size, collateral)?;
                positions_manager_utils.validate_liquidation(
                    self.ctx(),
                    account,
                    collateral_token,
                    index_token,
                    is_long,
                    true,
                )?;

                collateral
            };

            if is_long {
                let value = safe_sub_to_int(safe_sub(collateral, collateral)?, size_delta)?;
                self.update_guaranteed_usd(collateral_token, value)?;
            }
            let position = self.position(account, collateral_token, index_token, is_long)?;

            let price = self.get_price(index_token)?;
            evm::log(DecreasePosition {
                account,
                collateral_token,
                index_token,
                collateral_delta,
                size_delta,
                is_long,
                price,
                fee: safe_sub(usd_out, usd_out_after_fee)?,
                usd_out,
            });
            evm::log(UpdatePosition {
                size: position.size,
                collateral: position.collateral,
                average_price: position.average_price,
                entry_funding_rate: position.entry_funding_rate,
                reserve_amount: position.reserve_amount,
                realised_pnl: position.realised_pnl,
                mark_price: price,
            });
        } else {
            if is_long {
                self.update_guaranteed_usd(
                    collateral_token,
                    safe_sub_to_int(collateral, size_delta)?,
                )?;
            }

            let price = self.get_price(index_token)?;

            evm::log(DecreasePosition {
                account,
                collateral_token,
                index_token,
                collateral_delta,
                size_delta,
                is_long,
                price,
                fee: safe_sub(usd_out, usd_out_after_fee)?,
                usd_out,
            });
            evm::log(ClosePosition {
                size: position.size,
                collateral: position.collateral,
                average_price: position.average_price,
                entry_funding_rate: position.entry_funding_rate,
                reserve_amount: position.reserve_amount,
                realised_pnl: position.realised_pnl,
            });

            self.position_update(
                account,
                collateral_token,
                index_token,
                is_long,
                Position::default(),
            )?;
        }

        if !is_long {
            positions_manager.decrease_global_short_size(self.ctx(), index_token, size_delta)?;
        }

        if usd_out > U256::ZERO {
            if is_long {
                self.decrease_pool_amount(
                    collateral_token,
                    self.usd_to_token(collateral_token, usd_out)?,
                )?;
            }
            let amount_out_after_fees = self.usd_to_token(collateral_token, usd_out_after_fee)?;
            self.transfer_out(collateral_token, amount_out_after_fees, receiver)?;
            return Ok(amount_out_after_fees);
        }

        Ok(U256::ZERO)
    }
}
