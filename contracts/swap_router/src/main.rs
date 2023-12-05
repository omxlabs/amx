#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::call_context::GetCallContext;
use omx_interfaces::{
    erc20::{safe_transfer, safe_transfer_from},
    router::{RouterError, Swap, SwapPath},
    vault::{ISwapManager, IVault},
    weth::IWeth,
};
use stylus_sdk::{console, contract, evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct SwapRouter {
        bool initialized;

        address weth;
        address usdo;
        address vault;
        address swap_manager;
        address positions_router;
    }
}

impl SwapRouter {
    fn only_positions_router(&self) -> Result<(), RouterError> {
        if self.positions_router.get() != msg::sender() {
            return Err(RouterError::Forbidden);
        }

        Ok(())
    }

    fn transfer_eth_to_vault(&mut self) -> Result<(), Vec<u8>> {
        let weth = self.weth.get();
        let vault = self.vault.get();
        IWeth::new(weth).deposit(self.ctx().value(msg::value()), vault)?;

        Ok(())
    }

    fn transfer_out_eth(&mut self, amount_out: U256, receiver: Address) -> Result<(), Vec<u8>> {
        let weth = self.weth.get();
        IWeth::new(weth).withdraw(self, receiver, amount_out)?;

        Ok(())
    }

    fn swap_internal(
        &mut self,
        path: SwapPath,
        min_out: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        if path.is_direct() {
            return self.vault_swap_internal(path.token_in(), path.token_out(), min_out, receiver);
        }

        let (token_in, intermediate, token_out) = path.unwrap_indirect();
        let mid_out =
            self.vault_swap_internal(token_in, intermediate, U256::ZERO, contract::address())?;

        let vault = self.vault.get();
        safe_transfer(self.ctx(), intermediate, vault, mid_out)?;

        let amount_out = self.vault_swap_internal(intermediate, token_out, min_out, receiver)?;

        Ok(amount_out)
    }

    fn vault_swap_internal(
        &mut self,
        token_in: Address,
        token_out: Address,
        min_out: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        let swap_manager = ISwapManager::new(self.swap_manager.get());

        let usdo = self.usdo.get();

        let amount_out = if token_out == usdo {
            swap_manager.buy_usdo(self, token_in, receiver)?
        } else if token_in == usdo {
            swap_manager.sell_usdo(self, token_out, receiver)?
        } else {
            swap_manager.swap(self, token_in, token_out, receiver)?
        };

        if amount_out < min_out {
            return Err(RouterError::InsufficientAmountOut.into());
        }

        Ok(amount_out)
    }
}

#[external]
impl SwapRouter {
    pub fn init(
        &mut self,
        weth: Address,
        usdo: Address,
        vault: Address,
        swap_manager: Address,
        positions_router: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(RouterError::AlreadyInitialized.into());
        }

        self.weth.set(weth);
        self.usdo.set(usdo);
        self.vault.set(vault);
        self.swap_manager.set(swap_manager);
        self.positions_router.set(positions_router);

        self.initialized.set(true);

        Ok(())
    }

    pub fn direct_pool_deposit(&mut self, token: Address, amount: U256) -> Result<(), Vec<u8>> {
        let vault = self.vault.get();

        safe_transfer_from(self.ctx(), msg::sender(), token, vault, amount)?;

        IVault::new(vault).direct_pool_deposit(self.ctx(), token)?;

        Ok(())
    }

    pub fn swap_for_position(
        &mut self,
        path: Vec<Address>,
        min_out: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        self.only_positions_router()?;

        let path = SwapPath::from_arr(path)?;

        self.swap_internal(path, min_out, receiver)
    }

    pub fn swap(
        &mut self,
        path: Vec<Address>,
        amount_in: U256,
        min_out: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        let vault = self.vault.get();
        let path = SwapPath::from_arr(path)?;

        safe_transfer_from(self.ctx(), path.token_in(), msg::sender(), vault, amount_in)?;

        let amount_out = self.swap_internal(path, min_out, receiver)?;

        evm::log(Swap {
            account: msg::sender(),
            token_in: path.token_in(),
            token_out: path.token_out(),
            amount_in,
            amount_out,
        });

        Ok(())
    }

    #[payable]
    pub fn swap_eth_to_tokens(
        &mut self,
        path: Vec<Address>,
        min_out: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        let path = SwapPath::from_arr(path)?;

        self.transfer_eth_to_vault()?;

        let amount_out = self.swap_internal(path, min_out, receiver)?;

        evm::log(Swap {
            account: msg::sender(),
            token_in: path.token_in(),
            token_out: path.token_out(),
            amount_in: msg::value(),
            amount_out,
        });

        Ok(())
    }

    pub fn swap_to_eth(
        &mut self,
        token_in: Address,
        amount_in: U256,
        min_out: U256,
        receiver: Address,
    ) -> Result<(), Vec<u8>> {
        let vault = self.vault.get();
        safe_transfer_from(self.ctx(), token_in, msg::sender(), vault, amount_in)?;

        let amount_out = self.swap_internal(
            SwapPath::Direct {
                token_in,
                token_out: self.weth.get(),
            },
            min_out,
            contract::address(),
        )?;

        self.transfer_out_eth(amount_out, receiver)?;

        evm::log(Swap {
            account: msg::sender(),
            token_in,
            token_out: self.weth.get(),
            amount_in,
            amount_out,
        });

        Ok(())
    }
}
