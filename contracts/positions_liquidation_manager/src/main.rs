#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, I256, U256};
use omx_common::{call_context::GetCallContext, safe_sub, safe_sub_to_int, LIQUIDATION_FEE_USD};
use omx_interfaces::vault::{
    position::Position, validate, CollectMarginFees, IFeeManager, IFundingRateManager,
    IPositionsDecreaseManager, IPositionsManager, IPositionsManagerUtils, IVault, IVaultUtils,
    LiquidatePosition, VaultError,
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsLiquidationManager {
        bool initialized;

        address gov;
        address vault;
        address vault_utils;
        address fee_manager;
        address funding_rate_manager;
        address positions_manager;
        address positions_manager_utils;
        address positions_decrease_manager;

        mapping (address => bool) is_liquidator;
    }
}

impl PositionsLiquidationManager {
    fn only_initialized(&self) -> Result<(), Vec<u8>> {
        validate(self.initialized.get(), VaultError::NotInitialized)?;

        Ok(())
    }

    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    fn update_cumulative_funding_rate(&mut self, token: Address) -> Result<(), Vec<u8>> {
        IFundingRateManager::new(self.funding_rate_manager.get())
            .update_cumulative_funding_rate(self, token)?;

        Ok(())
    }

    fn increase_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).increase_pool_amount(self, token, amount)?;

        Ok(())
    }

    fn decrease_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_pool_amount(self, token, amount)?;

        Ok(())
    }

    fn decrease_reserved_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_reserved_amount(self, token, amount)?;

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

    fn usd_to_token(&self, token: Address, usd_amount: U256) -> Result<U256, Vec<u8>> {
        Ok(IVaultUtils::new(self.vault_utils.get()).usd_to_token(self, token, usd_amount)?)
    }

    fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_price(self, token)?)
    }

    fn update_guaranteed_usd(&mut self, token: Address, value: I256) -> Result<(), Vec<u8>> {
        Ok(IPositionsManager::new(self.positions_manager.get())
            .update_guaranteed_usd(self, token, value)?)
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
}

#[external]
impl PositionsLiquidationManager {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        vault: Address,
        vault_utils: Address,
        fee_manager: Address,
        funding_rate_manager: Address,
        positions_manager: Address,
        positions_manager_utils: Address,
        positions_decrease_manager: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.vault.set(vault);
        self.vault_utils.set(vault_utils);
        self.fee_manager.set(fee_manager);
        self.funding_rate_manager.set(funding_rate_manager);
        self.positions_manager.set(positions_manager);
        self.positions_manager_utils.set(positions_manager_utils);
        self.positions_decrease_manager
            .set(positions_decrease_manager);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_liquidator(&mut self, liquidator: Address, is_active: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_liquidator.insert(liquidator, is_active);

        Ok(())
    }

    pub fn liquidate_position(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        is_long: bool,
        fee_receiver: Address,
    ) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        self.update_cumulative_funding_rate(collateral_token)?;

        let positions_manager = IPositionsManager::new(self.positions_manager.get());
        let positions_manager_utils =
            IPositionsManagerUtils::new(self.positions_manager_utils.get());

        let position = self.position(account, collateral_token, index_token, is_long)?;
        validate(position.size > U256::ZERO, VaultError::ZeroSize)?;

        let (liquidation_state, margin_fees) = positions_manager_utils.validate_liquidation(
            self.ctx(),
            account,
            collateral_token,
            index_token,
            is_long,
            false,
        )?;
        validate(
            liquidation_state != U256::ZERO,
            VaultError::LiquidateValidPosition,
        )?;

        if liquidation_state == U256::from(2) {
            let positions_decrease_manager =
                IPositionsDecreaseManager::new(self.positions_decrease_manager.get());
            positions_decrease_manager.decrease_position(
                self.ctx(),
                account,
                collateral_token,
                index_token,
                U256::ZERO,
                position.size,
                is_long,
                account,
            )?;
            return Ok(());
        }

        let fee_tokens = self.usd_to_token(collateral_token, margin_fees)?;

        let fee_manager = IFeeManager::new(self.fee_manager.get());
        fee_manager.increase_fee_reserves(self.ctx(), collateral_token, fee_tokens)?;

        evm::log(CollectMarginFees {
            token: collateral_token,
            fee_usd: margin_fees,
            fee_tokens,
        });

        let position = self.position(account, collateral_token, index_token, is_long)?;
        self.decrease_reserved_amount(collateral_token, position.reserve_amount)?;
        if is_long {
            let delta = safe_sub_to_int(position.collateral, position.size)?;
            self.update_guaranteed_usd(collateral_token, delta)?;
            self.decrease_pool_amount(collateral_token, fee_tokens)?;
        }

        let marg_price = self.get_price(index_token)?;
        evm::log(LiquidatePosition {
            account,
            collateral_token,
            index_token,
            is_long,
            size: position.size,
            collateral: position.collateral,
            reserve_amount: position.reserve_amount,
            realised_pnl: position.realised_pnl,
            mark_price: marg_price,
        });

        if !is_long && margin_fees < position.collateral {
            let remaining_collateral = safe_sub(position.collateral, margin_fees)?;
            self.increase_pool_amount(
                collateral_token,
                self.usd_to_token(collateral_token, remaining_collateral)?,
            )?;
        }

        if !is_long {
            positions_manager.decrease_global_short_size(self.ctx(), index_token, position.size)?;
        }

        self.position_update(
            account,
            collateral_token,
            index_token,
            is_long,
            Position::default(),
        )?;

        // pay the fee receiver using the pool, we assume that in general the liquidated amount should
        // be sufficient to cover the liquidation fees
        self.decrease_pool_amount(
            collateral_token,
            self.usd_to_token(collateral_token, LIQUIDATION_FEE_USD)?,
        )?;
        self.transfer_out(
            collateral_token,
            self.usd_to_token(collateral_token, LIQUIDATION_FEE_USD)?,
            fee_receiver,
        )?;

        Ok(())
    }
}
