#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::call_context::GetCallContext;
use omx_interfaces::{
    router::RouterError,
    vault::{IPositionsDecreaseManager, IVault},
    weth::IWeth,
};
use stylus_sdk::{contract, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct PositionsDecreaseRouter {
        bool initialized;

        address weth;
        address vault;
        address positions_decrease_manager;
        address swap_router;
    }
}

impl PositionsDecreaseRouter {
    fn only_initialized(&self) -> Result<(), RouterError> {
        if !self.initialized.get() {
            return Err(RouterError::AlreadyInitialized);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn decrease_position_internal(
        &mut self,
        collateral_token: Address,
        index_token: Address,
        collateral_delta: U256,
        size_delta: U256,
        is_long: bool,
        receiver: Address,
        price: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_initialized()?;

        let vault = IVault::new(self.vault.get());

        let current_price = vault.get_price(self.ctx(), index_token)?;
        if is_long {
            if current_price < price {
                return Err(RouterError::LowMarkPrice.into());
            }
        } else if current_price > price {
            return Err(RouterError::HighMarkPrice.into());
        }

        let decrease_manager =
            IPositionsDecreaseManager::new(self.positions_decrease_manager.get());
        let amount_out = decrease_manager.decrease_position(
            self.ctx(),
            msg::sender(),
            collateral_token,
            index_token,
            collateral_delta,
            size_delta,
            is_long,
            receiver,
        )?;

        Ok(amount_out)
    }

    fn transfer_out_eth(&mut self, amount_out: U256, receiver: Address) -> Result<(), Vec<u8>> {
        let weth = self.weth.get();
        IWeth::new(weth).withdraw(self, receiver, amount_out)?;

        Ok(())
    }
}

#[external]
impl PositionsDecreaseRouter {
    pub fn init(
        &mut self,
        weth: Address,
        vault: Address,
        positions_decrease_manager: Address,
        swap_router: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(RouterError::AlreadyInitialized.into());
        }

        self.weth.set(weth);
        self.vault.set(vault);
        self.positions_decrease_manager
            .set(positions_decrease_manager);
        self.swap_router.set(swap_router);

        self.initialized.set(true);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn decrease_position(
        &mut self,
        collateral_token: Address,
        index_token: Address,
        collateral_delta: U256,
        size_delta: U256,
        is_long: bool,
        receiver: Address,
        price: U256,
    ) -> Result<U256, Vec<u8>> {
        self.decrease_position_internal(
            collateral_token,
            index_token,
            collateral_delta,
            size_delta,
            is_long,
            receiver,
            price,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn decrease_position_eth(
        &mut self,
        collateral_token: Address,
        index_token: Address,
        collateral_delta: U256,
        size_delta: U256,
        is_long: bool,
        receiver: Address,
        price: U256,
    ) -> Result<U256, Vec<u8>> {
        let amount_out = self.decrease_position_internal(
            collateral_token,
            index_token,
            collateral_delta,
            size_delta,
            is_long,
            contract::address(),
            price,
        )?;

        self.transfer_out_eth(amount_out, receiver)?;

        Ok(amount_out)
    }
}
