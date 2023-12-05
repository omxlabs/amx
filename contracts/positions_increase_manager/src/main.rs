#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, I256, U256};
use omx_common::{call_context::GetCallContext, safe_add, safe_sub, safe_sub_to_int};
use omx_interfaces::vault::{
    position::{validate_position, Position},
    validate, IFeeManager, IFundingRateManager, IPositionsManager, IPositionsManagerUtils, IVault,
    IVaultUtils, IncreasePosition, UpdatePosition, VaultError,
};
use stylus_sdk::{block, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsIncreaseManager {
        bool initialized;

        address gov;
        address vault;
        address vault_utils;
        address fee_manager;
        address funding_rate_manager;
        address increase_router;
        address positions_manager;
        address positions_manager_utils;
    }
}

impl PositionsIncreaseManager {
    fn only_initialized(&self) -> Result<(), Vec<u8>> {
        validate(self.initialized.get(), VaultError::NotInitialized)?;

        Ok(())
    }

    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    fn validate_router(&self, account: Address) -> Result<(), Vec<u8>> {
        if msg::sender() == account || msg::sender() == self.increase_router.get() {
            return Ok(());
        }

        Ok(())
    }

    fn update_guaranteed_usd_internal(
        &mut self,
        token: Address,
        value: I256,
    ) -> Result<(), Vec<u8>> {
        Ok(IPositionsManager::new(self.positions_manager.get())
            .update_guaranteed_usd(self, token, value)?)
    }

    fn update_cumulative_funding_rate(&mut self, token: Address) -> Result<(), Vec<u8>> {
        IFundingRateManager::new(self.funding_rate_manager.get())
            .update_cumulative_funding_rate(self, token)?;

        Ok(())
    }

    fn cumulative_funding_rate(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IFundingRateManager::new(self.funding_rate_manager.get())
            .cumulative_funding_rate(self, token)?)
    }

    fn increase_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).increase_pool_amount(self, token, amount)?;

        Ok(())
    }

    fn decrease_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_pool_amount(self, token, amount)?;

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

    fn usd_to_token(&self, token: Address, usd_amount: U256) -> Result<U256, Vec<u8>> {
        Ok(IVaultUtils::new(self.vault_utils.get()).usd_to_token(self, token, usd_amount)?)
    }
}

#[external]
impl PositionsIncreaseManager {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        vault: Address,
        vault_utils: Address,
        fee_manager: Address,
        funding_rate_manager: Address,
        increase_router: Address,
        positions_manager: Address,
        positions_manager_utils: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.vault.set(vault);
        self.vault_utils.set(vault_utils);
        self.fee_manager.set(fee_manager);
        self.funding_rate_manager.set(funding_rate_manager);
        self.increase_router.set(increase_router);
        self.positions_manager.set(positions_manager);
        self.positions_manager_utils.set(positions_manager_utils);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn increase_position(
        &mut self,
        account: Address,
        collateral_token: Address,
        index_token: Address,
        size_delta: U256,
        is_long: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        self.validate_router(account)?;

        let positions_manager = IPositionsManager::new(self.positions_manager.get());

        let positions_manager_utils =
            IPositionsManagerUtils::new(self.positions_manager_utils.get());
        let vault_utils = IVaultUtils::new(self.vault_utils.get());
        let vault = IVault::new(self.vault.get());

        vault_utils.validate_tokens(self.ctx(), collateral_token, index_token, is_long)?;

        self.update_cumulative_funding_rate(collateral_token)?;

        let mut position = self.position(account, collateral_token, index_token, is_long)?;

        let price = vault.get_price(self.ctx(), index_token)?;
        position.average_price = price;

        if position.size == U256::ZERO {
            self.position_update(account, collateral_token, index_token, is_long, position)?;
        }

        if position.size > U256::ZERO && size_delta > U256::ZERO {
            position.average_price = positions_manager_utils.get_next_average_price(
                self.ctx(),
                index_token,
                position.size,
                position.average_price,
                is_long,
                price,
                size_delta,
                position.last_increased_time,
            )?;
            self.position_update(account, collateral_token, index_token, is_long, position)?;
        }

        let fee = self.collect_margin_fees(
            collateral_token,
            size_delta,
            position.size,
            position.entry_funding_rate,
        )?;

        let collateral_delta = vault.transfer_in(self.ctx(), collateral_token)?;
        let collateral_delta_usd =
            vault_utils.token_to_usd(self.ctx(), collateral_token, collateral_delta)?;

        position.collateral = safe_sub(safe_add(position.collateral, collateral_delta_usd)?, fee)
            .map_err(|_| VaultError::CollateralLessThenFees)?;

        position.entry_funding_rate = self.cumulative_funding_rate(collateral_token)?;

        position.size = safe_add(position.size, size_delta)?;
        validate(position.size > U256::ZERO, VaultError::ZeroSize)?;

        position.last_increased_time = U256::from(block::timestamp());

        self.position_update(account, collateral_token, index_token, is_long, position)?;

        validate_position(position.size, position.collateral)?;

        positions_manager_utils.validate_liquidation(
            self.ctx(),
            account,
            collateral_token,
            index_token,
            is_long,
            true,
        )?;

        // reserve tokens to pay profits on the position
        let reserve_delta = self.usd_to_token(collateral_token, size_delta)?;
        position.reserve_amount = safe_add(position.reserve_amount, reserve_delta)?;

        self.position_update(account, collateral_token, index_token, is_long, position)?;

        vault.increase_reserved_amount(self.ctx(), collateral_token, reserve_delta)?;

        if is_long {
            // guaranteed_usd stores the sum of `position.size - position.collateral` for all positions
            // if a fee is charged on the collateral then guaranteed_usd should be increased by that fee amount
            // since `position.size - position.collateral` would have increased by `fee`
            self.update_guaranteed_usd_internal(
                collateral_token,
                safe_sub_to_int(safe_add(size_delta, fee)?, collateral_delta_usd)?,
            )?;
            // treat the deposited collateral as part of the pool
            self.increase_pool_amount(collateral_token, collateral_delta)?;
            // fees need to be deducted from the pool since fees are deducted from `position.collateral`
            // and collateral is treated as part of the pool
            self.decrease_pool_amount(collateral_token, self.usd_to_token(collateral_token, fee)?)?;
        } else {
            positions_manager.after_short_increase(self.ctx(), index_token, price, size_delta)?;
        }

        evm::log(IncreasePosition {
            account,
            collateral_token,
            index_token,
            collateral_delta: collateral_delta_usd,
            size_delta,
            is_long,
            price,
            fee,
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

        Ok(())
    }
}
