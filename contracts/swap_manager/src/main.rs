#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    call_context::GetCallContext, safe_add, safe_mul_ratio, safe_sub, MINT_BURN_FEE_BASIS_POINTS,
    PRICE_PRECISION, USDO_DECIMALS,
};
use omx_interfaces::{
    base_token::IBaseToken,
    vault::{
        validate, BuyUSDO, DecreaseUsdoAmount, IFeeManager, IFundingRateManager, IVault, SellUSDO,
        Swap, VaultError,
    },
};
use stylus_sdk::{console, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct SwapManager {
        bool initialized;

        address gov;
        address usdo;
        address vault;
        address fee_manager;
        address funding_rate_manager;

        /// buffer_amounts allows specification of an amount to exclude from swaps
        /// this can be used to ensure a certain amount of liquidity is available for leverage positions
        mapping (address => uint256) buffer_amounts;

        bool in_manager_mode;
        mapping (address => bool) is_manager;

        /// usdo_amounts tracks the amount of USDO debt for each whitelisted token
        mapping (address => uint256) usdo_amounts;

        /// max_usdo_amounts allows setting a max amount of USDO debt for a token
        mapping (address => uint256) max_usdo_amounts;
    }
}

impl SwapManager {
    fn only_gov(&self) -> Result<(), Vec<u8>> {
        validate(msg::sender() == self.gov.get(), VaultError::Forbidden)?;

        Ok(())
    }

    fn only_initialized(&self) -> Result<(), Vec<u8>> {
        validate(self.initialized.get(), VaultError::Forbidden)?;

        Ok(())
    }

    fn only_manager(&self) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        if self.in_manager_mode.get() {
            validate(self.is_manager.get(msg::sender()), VaultError::ManagerOnly)?;
        }

        Ok(())
    }

    fn validate_white_listed(&self, token: Address) -> Result<(), Vec<u8>> {
        validate(
            IVault::new(self.vault.get()).is_whitelisted(self, token)?,
            VaultError::TokenNotWhitelisted,
        )?;

        Ok(())
    }

    fn transfer_in(&mut self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).transfer_in(self, token)?)
    }

    fn transfer_out(
        &mut self,
        token: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        Ok(IVault::new(self.vault.get()).transfer_out(self, token, amount, receiver)?)
    }

    fn decrease_usdo_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        let value = self.usdo_amounts.get(token);

        // since USDO can be minted using multiple assets
        // it is possible for the USDO debt for a single asset to be less than zero
        // the USDO debt is capped to zero for this case
        if value <= amount {
            self.usdo_amounts.insert(token, U256::ZERO);
            evm::log(DecreaseUsdoAmount {
                amount: value,
                token,
            });
            return Ok(());
        }

        self.usdo_amounts.insert(token, safe_sub(value, amount)?);
        evm::log(DecreaseUsdoAmount { amount, token });
        Ok(())
    }

    fn increase_usdo_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        let usdo_amount = safe_add(self.usdo_amounts.get(token), amount)?;

        let max_usdo_amount = self.max_usdo_amounts.get(token);
        if max_usdo_amount != U256::ZERO {
            validate(
                usdo_amount <= max_usdo_amount,
                VaultError::MaxUsdoAmountExceeded,
            )?;
        }

        self.usdo_amounts.insert(token, usdo_amount);

        Ok(())
    }

    fn pool_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).pool_amount(self, token)?)
    }

    fn get_price(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_price(self, token)?)
    }

    fn token_decimals(&self, token: Address) -> Result<u8, Vec<u8>> {
        Ok(IVault::new(self.vault.get()).get_token_decimals(self, token)?)
    }

    fn update_cumulative_funding_rate(&mut self, token: Address) -> Result<(), Vec<u8>> {
        IFundingRateManager::new(self.funding_rate_manager.get())
            .update_cumulative_funding_rate(self, token)?;

        Ok(())
    }

    fn adjust_for_decimals(
        &self,
        amount: U256,
        token_div: Address,
        token_mul: Address,
    ) -> Result<U256, Vec<u8>> {
        let decimals_div = if token_div == self.usdo.get() {
            USDO_DECIMALS
        } else {
            self.token_decimals(token_div)?
        };
        let decimals_mul = if token_mul == self.usdo.get() {
            USDO_DECIMALS
        } else {
            self.token_decimals(token_mul)?
        };

        safe_mul_ratio(
            amount,
            U256::from(10).pow(U256::from(decimals_mul)),
            U256::from(10).pow(U256::from(decimals_div)),
        )
    }

    fn collect_swap_fees(
        &mut self,
        token: Address,
        amount: U256,
        fee_basis_points: U256,
    ) -> Result<U256, Vec<u8>> {
        Ok(IFeeManager::new(self.fee_manager.get()).collect_swap_fees(
            self,
            token,
            amount,
            fee_basis_points,
        )?)
    }

    fn increase_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).increase_pool_amount(self, token, amount)?;

        Ok(())
    }

    fn decrease_pool_amount(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        IVault::new(self.vault.get()).decrease_pool_amount(self, token, amount)?;

        Ok(())
    }
}

#[external]
impl SwapManager {
    pub fn init(
        &mut self,
        gov: Address,
        usdo: Address,
        vault: Address,
        fee_manager: Address,
        funding_rate_manager: Address,
    ) -> Result<(), Vec<u8>> {
        validate(!self.initialized.get(), VaultError::AlreadyInitialized)?;

        self.gov.set(gov);
        self.usdo.set(usdo);
        self.vault.set(vault);
        self.fee_manager.set(fee_manager);
        self.funding_rate_manager.set(funding_rate_manager);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_in_manager_mode(&mut self, in_manager_mode: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.in_manager_mode.set(in_manager_mode);

        Ok(())
    }

    pub fn set_manager(&mut self, account: Address, is_manager: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_manager.insert(account, is_manager);

        Ok(())
    }

    pub fn usdo_amount(&self, token: Address) -> Result<U256, Vec<u8>> {
        Ok(self.usdo_amounts.get(token))
    }

    pub fn buy_usdo(&mut self, token: Address, receiver: Address) -> Result<U256, Vec<u8>> {
        self.only_manager()?;

        self.validate_white_listed(token)?;

        let token_amount = self.transfer_in(token)?;
        validate(token_amount > U256::ZERO, VaultError::ZeroAmount)?;

        self.update_cumulative_funding_rate(token)?;

        let price = self.get_price(token)?;

        let usdo_amount = safe_mul_ratio(token_amount, price, PRICE_PRECISION)?;
        let usdo_amount = self.adjust_for_decimals(usdo_amount, token, self.usdo.get())?;
        validate(usdo_amount > U256::ZERO, VaultError::ZeroUsdoAmount)?;

        let fee_basis_points = MINT_BURN_FEE_BASIS_POINTS;

        let amount_after_fees = self.collect_swap_fees(token, token_amount, fee_basis_points)?;
        let mint_amount = safe_mul_ratio(amount_after_fees, price, PRICE_PRECISION)?;
        let usdo = self.usdo.get();
        let mint_amount = self.adjust_for_decimals(mint_amount, token, usdo)?;

        validate(mint_amount > U256::ZERO, VaultError::ZeroUsdoAmount)?;

        self.increase_usdo_amount(token, mint_amount)?;
        self.increase_pool_amount(token, amount_after_fees)?;

        IBaseToken::new(self.usdo.get()).mint(self, receiver, mint_amount)?;

        evm::log(BuyUSDO {
            account: receiver,
            token,
            token_amount,
            usdo_amount: mint_amount,
            fee_basis_points,
        });

        Ok(mint_amount)
    }

    pub fn sell_usdo(&mut self, token: Address, receiver: Address) -> Result<U256, Vec<u8>> {
        self.only_manager()?;
        self.validate_white_listed(token)?;

        let usdo_amount = self.transfer_in(self.usdo.get())?;
        validate(usdo_amount > U256::ZERO, VaultError::ZeroUsdoAmount)?;

        self.update_cumulative_funding_rate(token)?;

        let price = self.get_price(token)?;
        let redemption_amount = safe_mul_ratio(usdo_amount, PRICE_PRECISION, price)?;
        let redemption_amount =
            self.adjust_for_decimals(redemption_amount, self.usdo.get(), token)?;

        validate(
            redemption_amount > U256::ZERO,
            VaultError::ZeroRedemptionAmount,
        )?;

        self.decrease_usdo_amount(token, usdo_amount)?;
        self.decrease_pool_amount(token, redemption_amount)?;

        let vault = self.vault.get();
        let usdo = IBaseToken::new(self.usdo.get());
        usdo.burn(self.ctx(), vault, usdo_amount)?;

        // the transfer_in call increased the value of token_balances[usdo]
        // usually decreases in token balances are synced by calling transfer_out
        // however, for usdo, the tokens are burnt, so `update_token_balance`
        // should be manually called to record the decrease in tokens
        let vault = IVault::new(self.vault.get());
        vault.update_token_balance(self.ctx(), usdo.address)?;

        let fee_basis_points = MINT_BURN_FEE_BASIS_POINTS;

        let amount_out = self.collect_swap_fees(token, redemption_amount, fee_basis_points)?;
        validate(amount_out > U256::ZERO, VaultError::ZeroAmount)?;

        self.transfer_out(token, amount_out, receiver)?;

        evm::log(SellUSDO {
            account: receiver,
            token,
            usdo_amount,
            token_amount: amount_out,
            fee_basis_points,
        });

        Ok(amount_out)
    }

    pub fn swap(
        &mut self,
        token_in: Address,
        token_out: Address,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        self.validate_white_listed(token_in)?;
        self.validate_white_listed(token_out)?;

        validate(token_in != token_out, VaultError::SameToken)?;

        self.update_cumulative_funding_rate(token_in)?;
        self.update_cumulative_funding_rate(token_out)?;

        let amount_in = self.transfer_in(token_in)?;
        validate(amount_in > U256::ZERO, VaultError::ZeroAmount)?;

        let price_in = self.get_price(token_in)?;
        let price_out = self.get_price(token_out)?;

        let amount_out = safe_mul_ratio(amount_in, price_in, price_out)?;
        let amount_out = self.adjust_for_decimals(amount_out, token_in, token_out)?;

        // adjust usdoAmounts by the same usdo_amount as debt is shifted between the assets
        let usdo_amount = safe_mul_ratio(amount_in, price_in, PRICE_PRECISION)?;
        let usdo_amount = self.adjust_for_decimals(usdo_amount, token_in, self.usdo.get())?;

        let fee_manager = IFeeManager::new(self.fee_manager.get());
        let fee_basis_points =
            fee_manager.get_swap_fee_basis_points(self.ctx(), token_in, token_out)?;
        let amount_out_after_fees =
            self.collect_swap_fees(token_out, amount_out, fee_basis_points)?;

        self.increase_usdo_amount(token_in, usdo_amount)?;
        self.decrease_usdo_amount(token_out, usdo_amount)?;

        self.increase_pool_amount(token_in, amount_in)?;
        self.decrease_pool_amount(token_out, amount_out)?;

        validate(
            self.pool_amount(token_out)? >= self.buffer_amounts.get(token_out),
            VaultError::PoolLessThenBuffer,
        )?;

        self.transfer_out(token_out, amount_out_after_fees, receiver)?;

        evm::log(Swap {
            account: receiver,
            token_in,
            token_out,
            amount_in,
            amount_out,
            amount_out_after_fees,
            fee_basis_points,
        });

        Ok(amount_out_after_fees)
    }
}
