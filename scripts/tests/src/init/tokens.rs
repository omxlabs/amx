use std::sync::Arc;

use ethers::types::{Address, U256};

use crate::constants::{
    ATOM_DECIMALS, BNB_DECIMALS, BTC_DECIMALS, OSMO_DECIMALS, USDC_DECIMALS, USDT_DECIMALS,
};
use crate::contracts::distributor::{Distributor, DistributorInitArgs};
use crate::contracts::weth::{Weth, WethInitArgs};
use crate::contracts::yield_token::{YieldToken, YieldTokenInitArgs};
use crate::contracts::yield_tracker::{YieldTracker, YieldTrackerInitArgs};
use crate::contracts::{
    base_token::{BaseToken, BaseTokenInitArgs},
    erc20::{Erc20, Erc20InitArgs},
    ContractAddresses,
};
use crate::stylus_testing::provider::{TestClient, TestProvider};

/// Tokens contracts init helper
#[derive(Clone, Debug)]
pub struct TokensContractsInitArgs {
    pub gov: Address,
}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct TokensContracts {
    pub weth: Weth<TestClient>,
    pub dai: Erc20<TestClient>,
    pub btc: Erc20<TestClient>,
    pub atom: Erc20<TestClient>,
    pub osmo: Erc20<TestClient>,
    pub bnb: Erc20<TestClient>,
    pub usdt: Erc20<TestClient>,
    pub usdc: Erc20<TestClient>,
    pub omx: BaseToken<TestClient>,
    pub olp: BaseToken<TestClient>,
    pub es_omx: BaseToken<TestClient>,
    pub bn_omx: BaseToken<TestClient>,
    pub usdo: YieldToken<TestClient>,
    pub distributor: Distributor<TestClient>,
    pub yield_tracker: YieldTracker<TestClient>,
}

impl TokensContracts {
    pub async fn mint_weth(&self, to: Address, amount: U256) {
        let client = self.weth.client();
        client.mint_eth(client.address(), amount);
        self.weth.deposit(to).value(amount).await.unwrap();
    }

    pub async fn mint_dai(&self, to: Address, amount: U256) {
        self.dai.mint(to, amount).await.unwrap();
    }

    pub async fn mint_omx(&self, to: Address, amount: U256) {
        self.omx.mint(to, amount).await.unwrap();
    }

    pub async fn mint_olp(&self, to: Address, amount: U256) {
        self.olp.mint(to, amount).await.unwrap();
    }

    pub async fn mint_es_omx(&self, to: Address, amount: U256) {
        self.es_omx.mint(to, amount).await.unwrap();
    }

    pub async fn mint_bn_omx(&self, to: Address, amount: U256) {
        self.bn_omx.mint(to, amount).await.unwrap();
    }

    pub async fn mint_btc(&self, to: Address, amount: U256) {
        self.btc.mint(to, amount).await.unwrap();
    }

    pub async fn mint_atom(&self, to: Address, amount: U256) {
        self.atom.mint(to, amount).await.unwrap();
    }

    pub async fn mint_osmo(&self, to: Address, amount: U256) {
        self.osmo.mint(to, amount).await.unwrap();
    }

    pub async fn mint_bnb(&self, to: Address, amount: U256) {
        self.bnb.mint(to, amount).await.unwrap();
    }

    pub async fn mint_usdt(&self, to: Address, amount: U256) {
        self.usdt.mint(to, amount).await.unwrap();
    }

    pub async fn mint_usdc(&self, to: Address, amount: U256) {
        self.usdc.mint(to, amount).await.unwrap();
    }
}

impl TokensContractsInitArgs {
    pub async fn init(
        self,
        client: Arc<TestClient>,
        contracts: &ContractAddresses,
    ) -> TokensContracts {
        let usdo = YieldTokenInitArgs {
            name: "USD on OMX".to_string(),
            symbol: "USDO".to_string(),
            minter: contracts.vault.swap_manager,
            initial_supply: U256::zero(),
        }
        .init(client.clone(), contracts.tokens.usdo)
        .await;

        let omx = BaseTokenInitArgs {
            name: "OMX".to_string(),
            symbol: "OMX".to_string(),
            gov: self.gov,
        }
        .init(client.clone(), contracts.tokens.omx)
        .await;

        let olp = BaseTokenInitArgs {
            name: "OLP".to_string(),
            symbol: "OLP".to_string(),
            gov: self.gov,
        }
        .init(client.clone(), contracts.tokens.olp)
        .await;
        olp.set_in_private_transfer_mode(true).await.unwrap();
        olp.set_minter(contracts.staking.olp_manager, true)
            .await
            .unwrap();

        let bn_omx = BaseTokenInitArgs {
            name: "Binance OMX".to_string(),
            symbol: "BN-OMX".to_string(),
            gov: self.gov,
        }
        .init(client.clone(), contracts.tokens.bn_omx)
        .await;
        bn_omx
            .set_minter(contracts.staking.olp_manager, true)
            .await
            .unwrap();

        bn_omx.set_minter(self.gov, true).await.unwrap();
        bn_omx
            .set_minter(contracts.staking.reward_router, true)
            .await
            .unwrap();
        bn_omx
            .mint(
                contracts.staking.reward_router,
                U256::from_dec_str("15000000000000000000000000").unwrap(),
            )
            .await
            .unwrap();

        TokensContracts {
            weth: WethInitArgs {
                name: "Wrapped Ether".to_string(),
                symbol: "WETH".to_string(),
            }
            .init(client.clone(), contracts.tokens.weth)
            .await,

            dai: Erc20InitArgs {
                name: "Dai Stablecoin".to_string(),
                symbol: "DAI".to_string(),
                decimals: 18,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.dai)
            .await,

            bnb: Erc20InitArgs {
                name: "Binance Coin".to_string(),
                symbol: "BNB".to_string(),
                decimals: BNB_DECIMALS,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.bnb)
            .await,

            btc: Erc20InitArgs {
                name: "Bitcoin".to_string(),
                symbol: "BTC".to_string(),
                decimals: BTC_DECIMALS,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.btc)
            .await,

            atom: Erc20InitArgs {
                name: "Cosmos".to_string(),
                symbol: "ATOM".to_string(),
                decimals: ATOM_DECIMALS,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.atom)
            .await,

            osmo: Erc20InitArgs {
                name: "Osmosis".to_string(),
                symbol: "OSMO".to_string(),
                decimals: OSMO_DECIMALS,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.osmo)
            .await,

            usdt: Erc20InitArgs {
                name: "Tether USD".to_string(),
                symbol: "USDT".to_string(),
                decimals: USDT_DECIMALS,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.usdt)
            .await,

            usdc: Erc20InitArgs {
                name: "USD Coin".to_string(),
                symbol: "USDC".to_string(),
                decimals: USDC_DECIMALS,
                gov: self.gov,
            }
            .init(client.clone(), contracts.tokens.usdc)
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
            .init(client.clone(), contracts.tokens.es_omx)
            .await,

            distributor: DistributorInitArgs {}
                .init(client.clone(), contracts.tokens.distributor)
                .await,

            yield_tracker: YieldTrackerInitArgs {
                gov: self.gov,
                yield_token: contracts.tokens.usdo,
            }
            .init(client.clone(), contracts.tokens.yield_tracker)
            .await,
        }
    }
}
