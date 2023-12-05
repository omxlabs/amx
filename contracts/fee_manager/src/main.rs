#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    safe_add, safe_mul_ratio, safe_sub, BASIS_POINTS_DIVISOR, FUNDING_RATE_PRECISION,
    MARGIN_FEE_BASIS_POINTS, STABLE_SWAP_FEE_BASIS_POINTS, SWAP_FEE_BASIS_POINTS,
};
use omx_interfaces::{
    base_token::IBaseToken,
    vault::{
        validate, CollectMarginFees, CollectSwapFees, IFundingRateManager, IVault, VaultError,
    },
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct FeeManager {
        bool initialized;

        address gov;
        address usdo;
        address vault;
        address funding_rate_manager;
        address swap_manager;
        address positions_manager;
        address positions_manager_utils;
        address positions_increase_manager;
        address positions_decrease_manager;
        address positions_liquidation_manager;

        /// fee_reserves tracks the amount of fees per token
        mapping (address => uint256) fee_reserves;
    }
}

impl FeeManager {
    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(self.gov.get() == msg::sender(), VaultError::Forbidden)?;

        Ok(())
    }

    fn only_swap_manager(&self) -> Result<(), Vec<u8>> {
        validate(
            self.swap_manager.get() == msg::sender(),
            VaultError::Forbidden,
        )?;

        Ok(())
    }

    fn only_positions_manager(&self) -> Result<(), Vec<u8>> {
        validate(
            self.positions_manager.get() == msg::sender()
                || self.positions_manager_utils.get() == msg::sender()
                || self.positions_increase_manager.get() == msg::sender()
                || self.positions_decrease_manager.get() == msg::sender()
                || self.positions_liquidation_manager.get() == msg::sender(),
            VaultError::Forbidden,
        )?;

        Ok(())
    }

    fn is_stable(&self, token: Address) -> Result<bool, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).is_stable(self, token)?)
    }

    fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_price(self, token)?)
    }

    fn token_decimals(&self, token: Address) -> Result<u8, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_token_decimals(self, token)?)
    }

    fn token_weight(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).token_weight(self, token)?)
    }

    fn total_token_weights(&self) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).total_token_weights(self)?)
    }

    fn cumulative_funding_rate(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IFundingRateManager::new(self.funding_rate_manager.get())
            .cumulative_funding_rate(self, token)?)
    }

    fn token_to_usd(&self, token: Address, token_amount: U256) -> Result<U256, Vec<u8>> {
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

    fn usd_to_token(&self, token: Address, usd_amount: U256) -> Result<U256, Vec<u8>> {
        if usd_amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let price = self.get_price(token)?;
        let decimals = self.token_decimals(token)?;

        safe_mul_ratio(usd_amount, U256::from(10).pow(U256::from(decimals)), price)
    }

    fn transfer_out(
        &mut self,
        token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        Ok(IVault::new(self.vault.get()).transfer_out(self, token, amount, receiver)?)
    }
}

#[external]
impl FeeManager {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        usdo: Address,
        vault: Address,
        funding_rate_manager: Address,
        swap_manager: Address,
        positions_manager: Address,
        positions_manager_utils: Address,
        positions_increase_manager: Address,
        positions_decrease_manager: Address,
        positions_liquidation_manager: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.usdo.set(usdo);
        self.vault.set(vault);
        self.funding_rate_manager.set(funding_rate_manager);
        self.swap_manager.set(swap_manager);
        self.positions_manager.set(positions_manager);
        self.positions_manager_utils.set(positions_manager_utils);
        self.positions_increase_manager
            .set(positions_increase_manager);
        self.positions_decrease_manager
            .set(positions_decrease_manager);
        self.positions_liquidation_manager
            .set(positions_liquidation_manager);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn increase_fee_reserves(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.only_positions_manager()?;

        self.fee_reserves
            .insert(token, safe_add(self.fee_reserves.get(token), amount)?);

        Ok(())
    }

    pub fn get_fee_reserve(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.fee_reserves.get(token))
    }

    pub fn collect_swap_fees(
        &mut self,
        token: Address,
        amount: U256,
        fee_basis_points: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_swap_manager()?;

        let after_fee_amount = safe_mul_ratio(
            amount,
            safe_sub(BASIS_POINTS_DIVISOR, fee_basis_points)?,
            BASIS_POINTS_DIVISOR,
        )?;
        let fee_amount: ruint::Uint<256, 4> = safe_sub(amount, after_fee_amount)?;

        let prev_fee_reserve = self.fee_reserves.get(token);

        self.fee_reserves
            .insert(token, safe_add(prev_fee_reserve, fee_amount)?);

        evm::log(CollectSwapFees {
            token,
            fee_usd: self.token_to_usd(token, fee_amount)?,
            fee_tokens: fee_amount,
        });

        Ok(after_fee_amount)
    }

    pub fn get_position_fee(&self, size_delta: U256) -> Result<U256, Vec<u8>> {
        if size_delta == U256::ZERO {
            return Ok(U256::ZERO);
        }
        let after_fee_usd = safe_mul_ratio(
            size_delta,
            safe_sub(BASIS_POINTS_DIVISOR, MARGIN_FEE_BASIS_POINTS)?,
            BASIS_POINTS_DIVISOR,
        )?;
        safe_sub(size_delta, after_fee_usd)
    }

    pub fn get_funding_fee(
        &self,
        collateral_token: Address,
        size: U256,
        entry_funding_rate: U256,
    ) -> Result<U256, Vec<u8>> {
        if size == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let funding_rate = safe_sub(
            self.cumulative_funding_rate(collateral_token)?,
            entry_funding_rate,
        )?;
        if funding_rate == U256::ZERO {
            return Ok(U256::ZERO);
        }

        safe_mul_ratio(size, funding_rate, FUNDING_RATE_PRECISION)
    }

    pub fn collect_margin_fees(
        &mut self,
        collateral_token: Address,
        size_delta: U256,
        size: U256,
        entry_funding_rate: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_positions_manager()?;

        let fee_usd = self.get_position_fee(size_delta)?;

        let funding_fee = self.get_funding_fee(collateral_token, size, entry_funding_rate)?;
        let fee_usd = safe_add(fee_usd, funding_fee)?;

        let fee_tokens = self.usd_to_token(collateral_token, fee_usd)?;
        self.fee_reserves.insert(
            collateral_token,
            safe_add(self.fee_reserves.get(collateral_token), fee_tokens)?,
        );

        evm::log(CollectMarginFees {
            token: collateral_token,
            fee_usd,
            fee_tokens,
        });

        Ok(fee_usd)
    }

    pub fn withdraw_fees(&mut self, token: Address, receiver: Address) -> Result<U256, Vec<u8>> {
        self.only_gov()?;

        let amount = self.fee_reserves.get(token);
        if amount == U256::ZERO {
            return Ok(U256::ZERO);
        }

        self.fee_reserves.insert(token, U256::ZERO);
        self.transfer_out(token, amount, receiver)?;

        Ok(amount)
    }

    pub fn get_swap_fee_basis_points(
        &self,
        token_in: Address,
        token_out: Address,
    ) -> Result<U256, Vec<u8>> {
        if self.is_stable(token_in)? && self.is_stable(token_out)? {
            Ok(STABLE_SWAP_FEE_BASIS_POINTS)
        } else {
            Ok(SWAP_FEE_BASIS_POINTS)
        }
    }

    pub fn get_target_usdo_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        let supply = IBaseToken::new(self.usdo.get()).total_supply(self)?;

        if supply == U256::ZERO {
            return Ok(U256::ZERO);
        }

        let weight = self.token_weight(token)?;

        safe_mul_ratio(weight, supply, self.total_token_weights()?)
    }
}
