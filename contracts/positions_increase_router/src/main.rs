#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::call_context::GetCallContext;
use omx_interfaces::{
    erc20::{safe_transfer, safe_transfer_from},
    router::{CollateralPath, ISwapRouter, RouterError, SwapPath},
    vault::{IPositionsIncreaseManager, IVault},
    weth::IWeth,
};
use stylus_sdk::{contract, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsIncreaseRouter {
        bool initialized;

        address weth;
        address vault;
        address positions_increase_manager;
        address swap_router;
    }
}

impl PositionsIncreaseRouter {
    fn only_initialized(&self) -> Result<(), RouterError> {
        if !self.initialized.get() {
            return Err(RouterError::AlreadyInitialized);
        }

        Ok(())
    }

    fn increase_position_internal(
        &mut self,
        collateral_token: Address,
        index_token: Address,
        size_delta: U256,
        is_long: bool,
        price: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_initialized()?;

        let vault = IVault::new(self.vault.get());
        let increase_manager =
            IPositionsIncreaseManager::new(self.positions_increase_manager.get());

        let current_price = vault.get_price(self.ctx(), index_token)?;
        if is_long {
            if current_price > price {
                return Err(RouterError::HighMarkPrice.into());
            }
        } else if current_price < price {
            return Err(RouterError::LowMarkPrice.into());
        }

        increase_manager.increase_position(
            self.ctx(),
            msg::sender(),
            collateral_token,
            index_token,
            size_delta,
            is_long,
        )?;

        Ok(())
    }

    fn transfer_eth_to_vault(&mut self) -> Result<(), Vec<u8>> {
        let weth = self.weth.get();
        let vault = self.vault.get();

        IWeth::new(weth).deposit(self.ctx().value(msg::value()), vault)?;

        Ok(())
    }

    fn swap(&mut self, path: SwapPath, min_out: U256, receiver: Address) -> Result<U256, Vec<u8>> {
        Ok(ISwapRouter::new(self.swap_router.get()).swap_for_position(
            self,
            path.to_vec(),
            min_out,
            receiver,
        )?)
    }
}

#[external]
impl PositionsIncreaseRouter {
    pub fn init(
        &mut self,
        weth: Address,
        vault: Address,
        positions_increase_manager: Address,
        swap_router: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(RouterError::AlreadyInitialized.into());
        }

        self.weth.set(weth);
        self.vault.set(vault);
        self.positions_increase_manager
            .set(positions_increase_manager);
        self.swap_router.set(swap_router);

        self.initialized.set(true);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn increase_position(
        &mut self,
        path: Vec<Address>,
        index_token: Address,
        amount_in: U256,
        min_out: U256,
        size_delta: U256,
        is_long: bool,
        price: U256,
    ) -> Result<(), Vec<u8>> {
        let vault = self.vault.get();
        let path = CollateralPath::from_arr(path)?;

        if amount_in > U256::ZERO {
            safe_transfer_from(self.ctx(), path.token_in(), msg::sender(), vault, amount_in)?;
        }

        if path.is_path() && amount_in > U256::ZERO {
            let path = path.unwrap_path();
            let amount_out = self.swap(path, min_out, contract::address())?;
            safe_transfer(self.ctx(), path.token_out(), vault, amount_out)?;
        }

        self.increase_position_internal(path.token_out(), index_token, size_delta, is_long, price)?;

        Ok(())
    }

    #[payable]
    pub fn increase_position_eth(
        &mut self,
        path: Vec<Address>,
        index_token: Address,
        min_out: U256,
        size_delta: U256,
        is_long: bool,
        price: U256,
    ) -> Result<(), Vec<u8>> {
        let path = CollateralPath::from_arr(path)?;

        if path.token_in() != self.weth.get() {
            return Err(RouterError::InvalidTokenIn.into());
        }

        if msg::value() > U256::ZERO {
            self.transfer_eth_to_vault()?;
        }

        if path.is_path() && msg::value() > U256::ZERO {
            let path = path.unwrap_path();
            let vault = self.vault.get();

            let amount_out = self.swap(path, min_out, contract::address())?;
            safe_transfer(self.ctx(), path.token_out(), vault, amount_out)?;
        }

        self.increase_position_internal(path.token_out(), index_token, size_delta, is_long, price)?;

        Ok(())
    }
}
