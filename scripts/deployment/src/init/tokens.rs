use ethers::types::{Address, U256};

use crate::constants::{
    ATOM_DECIMALS, BNB_DECIMALS, BTC_DECIMALS, OSMO_DECIMALS, USDC_DECIMALS, USDT_DECIMALS,
};
use crate::contracts::distributor::{Distributor, DistributorInitArgs};
use crate::contracts::weth::{Weth, WethInitArgs};
use crate::contracts::yield_token::{YieldToken, YieldTokenInitArgs};
use crate::contracts::Erc20AddressData;
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

#[derive(Clone, Debug)]
pub struct Erc20TokenData {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub is_shortable: bool,
    pub is_stable: bool,
    pub pyth_id: String,
    pub contract: Erc20<LiveClient>,
}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct TokensContracts {
    pub weth: Weth<LiveClient>,
    pub erc20_tokens: Vec<Erc20TokenData>,
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

    pub async fn mint_erc20(&self, token: Address, to: Address, amount: U256) {
        let token = self
            .erc20_tokens
            .iter()
            .find(|t| t.contract.address() == token)
            .unwrap();
        send(token.contract.mint(to, amount)).await.unwrap();
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

            erc20_tokens: init_erc20_tokens(ctx, &contracts.tokens.erc20_tokens, self.gov).await,

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

async fn init_erc20_tokens(
    ctx: &DeployContext,
    tokens: &Vec<Erc20AddressData>,
    gov: Address,
) -> Vec<Erc20TokenData> {
    let mut result: Vec<Erc20TokenData> = Vec::with_capacity(tokens.len());

    for token in tokens {
        let contract = Erc20InitArgs {
            gov,
            decimals: token.decimals,
            name: token.name.clone(),
            symbol: token.symbol.clone(),
        }
        .init(ctx, token.address)
        .await;

        result.push(Erc20TokenData {
            name: token.name.clone(),
            symbol: token.symbol.clone(),
            decimals: token.decimals,
            is_shortable: token.is_shortable,
            is_stable: token.is_stable,
            pyth_id: token.pyth_id.clone(),
            contract,
        })
    }

    result
}
