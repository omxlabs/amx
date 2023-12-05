use ethers::types::{Address, U256};

use crate::constants::{
    ATOM_DECIMALS, BNB_DECIMALS, BTC_DECIMALS, OSMO_DECIMALS, USDC_DECIMALS, USDT_DECIMALS,
};
use crate::contracts::distributor::{Distributor, DistributorInitArgs};
use crate::contracts::weth::{Weth, WethInitArgs};
use crate::contracts::yield_token::{YieldToken, YieldTokenInitArgs};
use crate::{
    contracts::{
        base_token::{BaseToken, BaseTokenInitArgs},
        erc20::{Erc20, Erc20InitArgs},
        ContractAddresses, DeployContext, LiveClient,
    },
    utils::contract_call_helper::send,
};

/// Tokens contracts init helper
#[derive(Clone, Debug)]
pub struct TokensContractsInitArgs {
    pub gov: Address,
}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct TokensContracts {
    pub weth: Weth<LiveClient>,
    pub btc: Erc20<LiveClient>,
    pub atom: Erc20<LiveClient>,
    pub osmo: Erc20<LiveClient>,
    pub bnb: Erc20<LiveClient>,
    pub usdt: Erc20<LiveClient>,
    pub usdc: Erc20<LiveClient>,
    pub usdo: YieldToken<LiveClient>,
    pub olp: BaseToken<LiveClient>,
    pub omx: BaseToken<LiveClient>,
    pub es_omx: BaseToken<LiveClient>,
    pub bn_omx: BaseToken<LiveClient>,
    pub distributor: Distributor<LiveClient>,
}

impl TokensContracts {
    pub async fn mint_es_omx(&self, to: Address, amount: U256) {
        send(self.es_omx.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_olp(&self, to: Address, amount: U256) {
        send(self.olp.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_omx(&self, to: Address, amount: U256) {
        send(self.omx.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_btc(&self, to: Address, amount: U256) {
        send(self.btc.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_atom(&self, to: Address, amount: U256) {
        send(self.atom.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_osmo(&self, to: Address, amount: U256) {
        send(self.osmo.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_bnb(&self, to: Address, amount: U256) {
        send(self.bnb.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_usdt(&self, to: Address, amount: U256) {
        send(self.usdt.mint(to, amount)).await.unwrap();
    }

    pub async fn mint_usdc(&self, to: Address, amount: U256) {
        send(self.usdc.mint(to, amount)).await.unwrap();
    }
}

impl TokensContractsInitArgs {
    /// Initialize all vault contracts
    pub async fn init(self, ctx: &DeployContext, contracts: &ContractAddresses) -> TokensContracts {
        println!("initializing tokens contracts");

        let usdo = YieldTokenInitArgs {
            name: "USD on OMX".to_string(),
            symbol: "USDO".to_string(),
            initial_supply: U256::zero(),
            minter: contracts.vault.swap_manager,
        }
        .init(ctx, contracts.tokens.usdo)
        .await;

        let omx = BaseTokenInitArgs {
            name: "OMX".to_string(),
            symbol: "OMX".to_string(),
            gov: self.gov,
        }
        .init(ctx, contracts.tokens.omx)
        .await;

        let olp = BaseTokenInitArgs {
            name: "OLP".to_string(),
            symbol: "OLP".to_string(),
            gov: self.gov,
        }
        .init(ctx, contracts.tokens.olp)
        .await;
        send(olp.set_in_private_transfer_mode(true)).await.unwrap();
        send(olp.set_minter(contracts.staking.olp_manager, true))
            .await
            .unwrap();

        let bn_omx = BaseTokenInitArgs {
            name: "Binance OMX".to_string(),
            symbol: "BN-OMX".to_string(),
            gov: self.gov,
        }
        .init(ctx, contracts.tokens.bn_omx)
        .await;
        bn_omx
            .set_minter(contracts.staking.olp_manager, true)
            .await
            .unwrap();

        send(bn_omx.set_minter(self.gov, true)).await.unwrap();
        send(bn_omx.set_minter(contracts.staking.reward_router, true))
            .await
            .unwrap();
        send(bn_omx.mint(
            contracts.staking.reward_router,
            U256::from_dec_str("15000000000000000000000000").unwrap(),
        ))
        .await
        .unwrap();

        TokensContracts {
            weth: WethInitArgs {
                name: "Wrapped Ether".to_string(),
                symbol: "WETH".to_string(),
            }
            .init(ctx, contracts.tokens.weth)
            .await,

            bnb: Erc20InitArgs {
                name: "Binance Coin".to_string(),
                symbol: "BNB".to_string(),
                decimals: BNB_DECIMALS,
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.bnb)
            .await,

            btc: Erc20InitArgs {
                name: "Bitcoin".to_string(),
                symbol: "BTC".to_string(),
                decimals: BTC_DECIMALS,
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.btc)
            .await,

            atom: Erc20InitArgs {
                name: "Cosmos".to_string(),
                symbol: "ATOM".to_string(),
                decimals: ATOM_DECIMALS,
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.atom)
            .await,

            osmo: Erc20InitArgs {
                name: "Osmosis".to_string(),
                symbol: "OSMO".to_string(),
                decimals: OSMO_DECIMALS,
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.osmo)
            .await,

            usdt: Erc20InitArgs {
                name: "Tether USD".to_string(),
                symbol: "USDT".to_string(),
                decimals: USDT_DECIMALS,
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.usdt)
            .await,

            usdc: Erc20InitArgs {
                name: "USD Coin".to_string(),
                symbol: "USDC".to_string(),
                decimals: USDC_DECIMALS,
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.usdc)
            .await,

            usdo,
            omx,
            olp,
            bn_omx,
            es_omx: BaseTokenInitArgs {
                name: "Escrowed OMX".to_string(),
                symbol: "ES-OMX".to_string(),
                gov: self.gov,
            }
            .init(ctx, contracts.tokens.es_omx)
            .await,

            distributor: DistributorInitArgs {}
                .init(ctx, contracts.tokens.distributor)
                .await,
        }
    }
}
