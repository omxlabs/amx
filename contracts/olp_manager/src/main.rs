#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    call_context::GetCallContext, safe_add, safe_mul_ratio, safe_sub, OLP_MANAGER_COOLDOWN,
};
use omx_interfaces::{
    base_token::IBaseToken,
    erc20::{safe_transfer_from, IErc20},
    olp_manager::{AddLiquidity, IOlpManagerUtils, OlpManagerError, RemoveLiquidity},
    vault::ISwapManager,
};
use stylus_sdk::{block, contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct OlpManager {
        bool initialized;

        address gov;

        address vault;
        address swap_manager;
        address positions_manager;
        address shorts_tracker;
        address usdo;
        address olp;
        address utils;

        bool in_private_mode;

        mapping (address => uint256) last_added_at;

        mapping (address => bool) is_handler;
    }
}

impl OlpManager {
    fn only_gov(&self) -> Result<(), OlpManagerError> {
        if self.gov.get() != msg::sender() {
            return Err(OlpManagerError::Forbidden);
        }

        Ok(())
    }

    fn only_handler(&self) -> Result<(), OlpManagerError> {
        if !self.is_handler.get(msg::sender()) {
            return Err(OlpManagerError::Forbidden);
        }

        Ok(())
    }

    fn only_initialized(&self) -> Result<(), OlpManagerError> {
        if !self.initialized.get() {
            return Err(OlpManagerError::NotInitialized);
        }

        Ok(())
    }

    fn get_aum_in_usdo(&self) -> Result<U256, Vec<u8>> {
        Ok(IOlpManagerUtils::new(self.utils.get()).get_aum_in_usdo(self)?)
    }

    fn remove_liquidity_internal(
        &mut self,
        account: Address,
        token_out: Address,
        olp_amount: U256,
        min_amount: U256,
        recipient: Address,
    ) -> Result<U256, Vec<u8>> {
        if olp_amount == U256::ZERO {
            return Err(OlpManagerError::OlpZeroAmount.into());
        }
        if safe_add(self.last_added_at.get(account), OLP_MANAGER_COOLDOWN)?
            > U256::from(block::timestamp())
        {
            return Err(OlpManagerError::CooldownNotPassed.into());
        }

        let olp = IBaseToken::new(self.olp.get());
        let usdo = IBaseToken::new(self.usdo.get());
        let vault = self.vault.get();

        // calculate aum before sell_usdo
        let aum_in_usdo = self.get_aum_in_usdo()?;
        let olp_supply = olp.total_supply(self.ctx())?;

        let usdo_amount = safe_mul_ratio(olp_amount, aum_in_usdo, olp_supply)?;
        let usdo_balance = usdo.balance_of(self.ctx(), contract::address())?;

        if usdo_amount > usdo_balance {
            usdo.mint(
                self.ctx(),
                contract::address(),
                safe_sub(usdo_amount, usdo_balance)?,
            )?;
        }

        olp.burn(self.ctx(), account, olp_amount)?;

        usdo.transfer(self.ctx(), vault, usdo_amount)?;

        let swap_manager = ISwapManager::new(self.swap_manager.get());
        let amount_out = swap_manager.sell_usdo(self.ctx(), token_out, recipient)?;

        if amount_out < min_amount {
            return Err(OlpManagerError::InsufficientOutput { amount_out }.into());
        }

        evm::log(RemoveLiquidity {
            account,
            token: token_out,
            olp_amount,
            aum_in_usdo,
            olp_supply,
            usdo_amount,
            amount_out,
        });

        Ok(amount_out)
    }

    pub fn add_liquidity_internal(
        &mut self,
        funding_account: Address,
        account: Address,
        token: Address,
        amount: U256,
        min_usdo: U256,
        min_olp: U256,
    ) -> Result<U256, Vec<u8>> {
        if amount == U256::ZERO {
            return Err(OlpManagerError::TokenZeroAmount.into());
        }

        let olp = IBaseToken::new(self.olp.get());
        let token = IErc20::new(token);
        let swap_manager = ISwapManager::new(self.swap_manager.get());
        let vault = self.vault.get();

        // calculate aum before buy_usdo
        let aum_in_usdo = self.get_aum_in_usdo()?;

        let olp_supply = olp.total_supply(self.ctx())?;

        safe_transfer_from(self.ctx(), token.address, funding_account, vault, amount)?;

        let usdo_amount = swap_manager.buy_usdo(self.ctx(), token.address, contract::address())?;

        if usdo_amount < min_usdo {
            return Err(OlpManagerError::InsufficientUsdoOutput {
                amount_out: usdo_amount,
            }
            .into());
        }

        let mint_amount = if aum_in_usdo == U256::ZERO {
            usdo_amount
        } else {
            safe_mul_ratio(usdo_amount, olp_supply, aum_in_usdo)?
        };

        if mint_amount < min_olp {
            return Err(OlpManagerError::InsufficientOlpOutput {
                amount_out: mint_amount,
            }
            .into());
        }

        olp.mint(self.ctx(), account, mint_amount)?;

        self.last_added_at
            .insert(account, U256::from(block::timestamp()));

        evm::log(AddLiquidity {
            account,
            token: token.address,
            amount,
            aum_in_usdo,
            olp_supply,
            usdo_amount,
            mint_amount,
        });

        Ok(mint_amount)
    }
}

#[external]
impl OlpManager {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        olp_manager_utils: Address,
        vault: Address,
        swap_manager: Address,
        positions_manager: Address,
        shorts_tracker: Address,
        usdo: Address,
        olp: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(OlpManagerError::AlreadyInitialized.into());
        }

        self.gov.set(gov);

        self.utils.set(olp_manager_utils);
        self.vault.set(vault);
        self.swap_manager.set(swap_manager);
        self.positions_manager.set(positions_manager);
        self.shorts_tracker.set(shorts_tracker);
        self.usdo.set(usdo);
        self.olp.set(olp);

        self.initialized.set(true);

        Ok(())
    }

    pub fn is_handler(&self, account: Address) -> Result<bool, Vec<u8>> {
        Ok(self.is_handler.get(account))
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_in_private_mode(&mut self, in_private_mode: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.in_private_mode.set(in_private_mode);

        Ok(())
    }

    pub fn set_handler(&mut self, handler: Address, is_active: bool) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.is_handler.insert(handler, is_active);

        Ok(())
    }

    pub fn add_liquidity(
        &mut self,
        token: Address,
        amount: U256,
        min_usdo: U256,
        min_olp: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_initialized()?;

        if self.in_private_mode.get() {
            return Err(OlpManagerError::ActionNotEnabled.into());
        }

        self.add_liquidity_internal(
            msg::sender(),
            msg::sender(),
            token,
            amount,
            min_usdo,
            min_olp,
        )
    }

    pub fn add_liquidity_for_account(
        &mut self,
        funding_account: Address,
        account: Address,
        token: Address,
        amount: U256,
        min_usdo: U256,
        min_olp: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_handler()?;

        self.add_liquidity_internal(funding_account, account, token, amount, min_usdo, min_olp)
    }

    pub fn remove_liquidity(
        &mut self,
        token_out: Address,
        olp_amount: U256,
        min_amount: U256,
        recipient: Address,
    ) -> Result<U256, Vec<u8>> {
        self.only_initialized()?;

        if self.in_private_mode.get() {
            return Err(OlpManagerError::ActionNotEnabled.into());
        }

        self.remove_liquidity_internal(msg::sender(), token_out, olp_amount, min_amount, recipient)
    }

    pub fn remove_liquidity_for_account(
        &mut self,
        account: Address,
        token_out: Address,
        olp_amount: U256,
        min_amount: U256,
        recipient: Address,
    ) -> Result<U256, Vec<u8>> {
        self.only_handler()?;

        self.remove_liquidity_internal(account, token_out, olp_amount, min_amount, recipient)
    }
}
