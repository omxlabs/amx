use std::{fmt::Display, sync::Arc};

use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::Address,
};
use serde::Serialize;

use crate::{
    constants::ARTIFACTS_DIR, deploy::deploy_contract, utils::client::client_from_str_key,
};

pub mod base_token;
pub mod bonus_distributor;
pub mod distributor;
pub mod erc20;
pub mod fee_manager;
pub mod funding_rate_manager;
pub mod olp_manager;
pub mod olp_manager_utils;
pub mod orderbook_increase;
pub mod orderbook_swap;
pub mod positions_decrease_manager;
pub mod positions_decrease_router;
pub mod positions_increase_manager;
pub mod positions_increase_router;
pub mod positions_liquidation_manager;
pub mod positions_manager;
pub mod positions_manager_utils;
pub mod reward_distributor;
pub mod reward_router;
pub mod reward_tracker;
pub mod reward_tracker_staking;
pub mod shorts_tracker;
pub mod swap_manager;
pub mod swap_router;
pub mod vault;
pub mod vault_price_feed;
pub mod vault_utils;
pub mod weth;
pub mod yield_token;
pub mod yield_tracker;

pub type Client<P> = SignerMiddleware<Provider<P>, LocalWallet>;

pub type LiveClient = SignerMiddleware<Provider<Http>, LocalWallet>;

#[derive(Clone, Debug)]
pub struct DeployContext {
    endpoint: String,
    client: Arc<LiveClient>,
    key_path: String,
}

impl DeployContext {
    pub async fn new(endpoint: impl AsRef<str>, key_path: impl AsRef<str>) -> Self {
        Self {
            endpoint: endpoint.as_ref().to_string(),
            key_path: key_path.as_ref().to_string(),
            client: client_from_str_key(key_path, endpoint).await,
        }
    }

    pub fn client(&self) -> Arc<LiveClient> {
        self.client.clone()
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub fn key_path(&self) -> String {
        self.key_path.clone()
    }
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct OrderbookAddresses {
    pub swap: Address,
    pub increase: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct StakingAddresses {
    pub shorts_tracker: Address,
    pub reward_router: Address,
    pub olp_manager: Address,
    pub olp_manager_utils: Address,

    pub staked_omx_tracker: Address,
    pub staked_omx_tracker_staking: Address,
    pub staked_omx_distributor: Address,
    pub bonus_omx_tracker: Address,
    pub bonus_omx_tracker_staking: Address,
    pub bonus_omx_distributor: Address,
    pub fee_omx_tracker: Address,
    pub fee_omx_tracker_staking: Address,
    pub fee_omx_distributor: Address,
    pub fee_olp_tracker: Address,
    pub fee_olp_tracker_staking: Address,
    pub fee_olp_distributor: Address,
    pub staked_olp_tracker: Address,
    pub staked_olp_tracker_staking: Address,
    pub staked_olp_distributor: Address,
}

#[derive(Clone, Debug, Serialize)]
pub struct Erc20AddressData {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub address: Address,
    pub pyth_id: String,
    pub is_shortable: bool,
    pub is_stable: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct TokensAddresses {
    pub weth: Address,
    pub usdo: Address,
    pub es_omx: Address,
    pub bn_omx: Address,
    pub olp: Address,
    pub omx: Address,
    pub distributor: Address,

    pub erc20_tokens: Vec<Erc20AddressData>,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct VaultAddresses {
    pub fee_manager: Address,
    pub funding_rate_manager: Address,
    pub positions_manager: Address,
    pub positions_manager_utils: Address,
    pub positions_decrease_manager: Address,
    pub positions_increase_manager: Address,
    pub positions_liquidation_manager: Address,
    pub swap_manager: Address,
    pub vault: Address,
    pub vault_utils: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct RouterAddresses {
    pub positions_decrease: Address,
    pub positions_increase: Address,
    pub swap: Address,
}

#[derive(Clone, Debug, Serialize)]
pub struct ContractAddresses {
    pub vault_price_feed: Address,

    pub router: RouterAddresses,
    pub vault: VaultAddresses,
    pub tokens: TokensAddresses,
    pub orderbook: OrderbookAddresses,
    pub staking: StakingAddresses,
}

fn get_contract_path(contract: impl Display) -> String {
    format!("{}/omx_{}.wasm", ARTIFACTS_DIR, contract)
}

impl ContractAddresses {
    /// Deploy all contracts
    ///
    /// **NOTE**: This function only deploys contracts, you may also need to call `init` functions
    pub async fn deploy_contracts(ctx: &DeployContext) -> Self {
        let deploy = |name: &str| -> Address {
            deploy_contract(get_contract_path(name), &ctx.key_path, &ctx.endpoint)
        };

        Self {
            vault_price_feed: deploy("vault_price_feed"),

            staking: StakingAddresses {
                reward_router: deploy("reward_router"),
                shorts_tracker: deploy("shorts_tracker"),
                olp_manager: deploy("olp_manager"),
                olp_manager_utils: deploy("olp_manager_utils"),
                bonus_omx_distributor: deploy("bonus_distributor"),
                bonus_omx_tracker: deploy("reward_tracker"),
                bonus_omx_tracker_staking: deploy("reward_tracker_staking"),
                fee_olp_distributor: deploy("reward_distributor"),
                fee_olp_tracker: deploy("reward_tracker"),
                fee_olp_tracker_staking: deploy("reward_tracker_staking"),
                fee_omx_distributor: deploy("reward_distributor"),
                fee_omx_tracker: deploy("reward_tracker"),
                fee_omx_tracker_staking: deploy("reward_tracker_staking"),
                staked_olp_distributor: deploy("reward_distributor"),
                staked_olp_tracker: deploy("reward_tracker"),
                staked_olp_tracker_staking: deploy("reward_tracker_staking"),
                staked_omx_distributor: deploy("reward_distributor"),
                staked_omx_tracker: deploy("reward_tracker"),
                staked_omx_tracker_staking: deploy("reward_tracker_staking"),
            },

            vault: VaultAddresses {
                fee_manager: deploy("fee_manager"),
                funding_rate_manager: deploy("funding_rate_manager"),
                positions_decrease_manager: deploy("positions_decrease_manager"),
                positions_increase_manager: deploy("positions_increase_manager"),
                positions_liquidation_manager: deploy("positions_liquidation_manager"),
                positions_manager: deploy("positions_manager"),
                positions_manager_utils: deploy("positions_manager_utils"),
                swap_manager: deploy("swap_manager"),
                vault: deploy("vault"),
                vault_utils: deploy("vault_utils"),
            },

            router: RouterAddresses {
                swap: deploy("swap_router"),
                positions_decrease: deploy("positions_decrease_router"),
                positions_increase: deploy("positions_increase_router"),
            },

            tokens: TokensAddresses {
                usdo: deploy("yield_token"),
                weth: deploy("weth"),
                olp: deploy("base_token"),
                omx: deploy("base_token"),
                bn_omx: deploy("base_token"),
                es_omx: deploy("base_token"),
                distributor: deploy("time_distributor"),

                erc20_tokens: deploy_tokens(ctx),
            },

            orderbook: OrderbookAddresses {
                swap: deploy("orderbook_swap"),
                increase: deploy("orderbook_increase"),
            },
        }
    }
}

fn deploy_tokens(ctx: &DeployContext) -> Vec<Erc20AddressData> {
    let tokens_config = vec![
        (
            "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a",
            6,
            "usdc",
            "usdc",
            false,
            true,
        ),
        (
            "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
            8,
            "btc",
            "btc",
            true,
            false,
        ),
        (
            "5bc91f13e412c07599167bae86f07543f076a638962b8d6017ec19dab4a82814",
            18,
            "busd",
            "busd",
            false,
            true,
        ),
        (
            "b00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819",
            6,
            "atom",
            "atom",
            true,
            false,
        ),
        (
            "b00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819",
            6,
            "usdt",
            "usdt",
            false,
            true,
        ),
        (
            "2f95862b045670cd22bee3114c39763a4a08beeb663b145d283c31d7d1101c4f",
            8,
            "bnb",
            "bnb",
            true,
            false,
        ),
        (
            "dcef50dd0a4cd2dcc17e45df1676dcb336a11a61c69df7a0299b0150c672d25c",
            6,
            "doge",
            "doge",
            true,
            false,
        ),
        (
            "5de33a9112c2b700b8d30b8a3402c103578ccfa2765696471cc672bd5cf6ac52",
            18,
            "matic",
            "matic",
            true,
            false,
        ),
        (
            "ca3eed9b267293f6595901c734c7525ce8ef49adafe8284606ceb307afa2ca5b",
            10,
            "dot",
            "dot",
            true,
            false,
        ),
        (
            "60144b1d5c9e9851732ad1d9760e3485ef80be39b984f6bf60f82b28a2b7f126",
            18,
            "axl",
            "axl",
            true,
            false,
        ),
        (
            "2a01deaec9e51a579277b34b122399984d0bbf57e2458a7e42fecd2829867a0d",
            18,
            "ada",
            "ada",
            true,
            false,
        ),
        (
            "2b9ab1e972a281585084148ba1389800799bd4be63b957507db1349314e47445",
            18,
            "aave",
            "aave",
            true,
            false,
        ),
        (
            "15add95022ae13563a11992e727c91bdb6b55bc183d9d747436c80a483d8c864",
            18,
            "ape",
            "ape",
            true,
            false,
        ),
        (
            "03ae4db29ed4ae33d323568895aa00337e658e348b37509f5372ae51f0af00d5",
            18,
            "apt",
            "apt",
            true,
            false,
        ),
        (
            "3fa4252848f9f0a1480be62745a4629d9eb1322aebab8a791e344b3b9c1adcf5",
            18,
            "arb",
            "arb",
            true,
            false,
        ),
        (
            "93da3352f9f1d105fdfe4971cfa80e9dd777bfc5d0f683ebb6e1294b92137bb7",
            18,
            "avax",
            "avax",
            true,
            false,
        ),
        (
            "3dd2b63686a450ec7290df3a1e0b583c0481f651351edfa7636f39aed55cf8a3",
            18,
            "bch",
            "bch",
            true,
            false,
        ),
        (
            "856aac602516addee497edf6f50d39e8c95ae5fb0da1ed434a8c2ab9c3e877e9",
            18,
            "blur",
            "blur",
            true,
            false,
        ),
        (
            "b44565b8b9b39ab2f4ba792f1c8f8aa8ef7d780e709b191637ef886d96fd1472",
            18,
            "bsv",
            "bsv",
            true,
            false,
        ),
        (
            "972776d57490d31c32279c16054e5c01160bd9a2e6af8b58780c82052b053549",
            18,
            "canto",
            "canto",
            true,
            false,
        ),
        (
            "8879170230c9603342f3837cf9a8e76c61791198fb1271bb2552c9af7b33c933",
            18,
            "cfx",
            "cfx",
            true,
            false,
        ),
        (
            "4a8e42861cabc5ecb50996f92e7cfa2bce3fd0a2423b0c44c9b423fb2bd25478",
            18,
            "comp",
            "comp",
            true,
            false,
        ),
        (
            "a19d04ac696c7a6616d291c7e5d1377cc8be437c327b75adb5dc1bad745fcae8",
            18,
            "crv",
            "crv",
            true,
            false,
        ),
        (
            "6489800bb8974169adfe35937bf6736507097d13c190d760c557108c7e93a81b",
            18,
            "dydx",
            "dydx",
            true,
            false,
        ),
        (
            "b98e7ae8af2d298d2651eb21ab5b8b5738212e13efb43bd0dfbce7a74ba4b5d0",
            18,
            "fet",
            "fet",
            true,
            false,
        ),
        (
            "150ac9b959aee0051e4091f0ef5216d941f590e1c5e7f91cf7635b5c11628c0e",
            18,
            "fil",
            "fil",
            true,
            false,
        ),
        (
            "5c6c0d2386e3352356c3ab84434fafb5ea067ac2678a38a338c4a69ddc4bdb0c",
            18,
            "ftm",
            "ftm",
            true,
            false,
        ),
        (
            "6c75e52531ec5fd3ef253f6062956a8508a2f03fa0a209fb7fbc51efd9d35f88",
            18,
            "ftt",
            "ftt",
            true,
            false,
        ),
        (
            "735f591e4fed988cd38df74d8fcedecf2fe8d9111664e0fd500db9aa78b316b1",
            18,
            "fxs",
            "fxs",
            true,
            false,
        ),
        (
            "0781209c28fda797616212b7f94d77af3a01f3e94a5d421760aef020cf2bcb51",
            18,
            "gala",
            "gala",
            true,
            false,
        ),
        (
            "baa284eaf23edf975b371ba2818772f93dbae72836bbdea28b07d40f3cf8b485",
            18,
            "gmt",
            "gmt",
            true,
            false,
        ),
        (
            "b962539d0fcb272a494d65ea56f94851c2bcf8823935da05bd628916e2e9edbf",
            18,
            "gmx",
            "gmx",
            true,
            false,
        ),
        (
            "941320a8989414874de5aa2fc340a75d5ed91fdff1613dd55f83844d52ea63a2",
            18,
            "imx",
            "imx",
            true,
            false,
        ),
        (
            "7a5bc1d2b56ad029048cd63964b3ad2776eadf812edc1a43a31406cb54bff592",
            18,
            "inj",
            "inj",
            true,
            false,
        ),
        (
            "b43660a5f790c69354b0729a5ef9d50d68f1df92107540210b9cccba1f947cc2",
            18,
            "jto",
            "jto",
            true,
            false,
        ),
        (
            "c63e2a7f37a04e5e614c07238bedb25dcc38927fba8fe890597a593c0b2fa4ad",
            18,
            "ldo",
            "ldo",
            true,
            false,
        ),
        (
            "8ac0c70fff57e9aefdf5edf44b51d62c2d433653cbb2cf5cc06bb115af04d221",
            18,
            "link",
            "link",
            true,
            false,
        ),
        (
            "6e3f3fa8253588df9326580180233eb791e03b443a3ba7a1d892e73874e19a54",
            18,
            "ltc",
            "ltc",
            true,
            false,
        ),
        (
            "cd2cee36951a571e035db0dfad138e6ecdb06b517cc3373cd7db5d3609b7927c",
            18,
            "meme",
            "meme",
            true,
            false,
        ),
        (
            "e322f437708e16b033d785fceb5c7d61c94700364281a10fabc77ca20ef64bf1",
            18,
            "mina",
            "mina",
            true,
            false,
        ),
        (
            "9375299e31c0deb9c6bc378e6329aab44cb48ec655552a70d4b9050346a30378",
            18,
            "mkr",
            "mkr",
            true,
            false,
        ),
        (
            "c415de8d2eba7db216527dff4b60e8f3a5311c740dadb233e13e12547e226750",
            18,
            "near",
            "near",
            true,
            false,
        ),
        (
            "a8e6517966a52cb1df864b2764f3629fde3f21d2b640b5c572fcd654cbccd65e",
            18,
            "ntrn",
            "ntrn",
            true,
            false,
        ),
        (
            "385f64d993f7b77d8182ed5003d97c60aa3361f3cecfe711544d2d59165e9bdf",
            18,
            "op",
            "op",
            true,
            false,
        ),
        (
            "193c739db502aadcef37c2589738b1e37bdb257d58cf1ab3c7ebc8e6df4e3ec0",
            18,
            "ordi",
            "ordi",
            true,
            false,
        ),
        (
            "9a4df90b25497f66b1afb012467e316e801ca3d839456db028892fe8c70c8016",
            18,
            "pendle",
            "pendle",
            true,
            false,
        ),
        (
            "0bbf28e9a841a1cc788f6a361b17ca072d0ea3098a1e5df1c3922d06719579ff",
            18,
            "pyth",
            "pyth",
            true,
            false,
        ),
        (
            "c8cf45412be4268bef8f76a8b0d60971c6e57ab57919083b8e9f12ba72adeeb6",
            18,
            "rdnt",
            "rdnt",
            true,
            false,
        ),
        (
            "2f2d17abbc1e781bd87b4a5d52c8b2856886f5c482fa3593cebf6795040ab0b6",
            18,
            "rlb",
            "rlb",
            true,
            false,
        ),
        (
            "ab7347771135fc733f8f38db462ba085ed3309955f42554a14fa13e855ac0e2f",
            18,
            "rndr",
            "rndr",
            true,
            false,
        ),
        (
            "5fcf71143bb70d41af4fa9aa1287e2efd3c5911cee59f909f915c9f61baacb1e",
            18,
            "rune",
            "rune",
            true,
            false,
        ),
        (
            "53614f1cb0c031d4af66c04cb9c756234adad0e1cee85303795091499a4084eb",
            18,
            "sei",
            "sei",
            true,
            false,
        ),
        (
            "39d020f60982ed892abbcd4a06a276a9f9b7bfbce003204c110b6e488f502da3",
            18,
            "snx",
            "snx",
            true,
            false,
        ),
        (
            "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
            18,
            "sol",
            "sol",
            true,
            false,
        ),
        (
            "ec7a775f46379b5e943c3526b1c8d54cd49749176b0b98e02dde68d1bd335c17",
            18,
            "stx",
            "stx",
            true,
            false,
        ),
        (
            "23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744",
            18,
            "sui",
            "sui",
            true,
            false,
        ),
        (
            "26e4f737fde0263a9eea10ae63ac36dcedab2aaf629261a994e1eeb6ee0afe53",
            18,
            "sushi",
            "sushi",
            true,
            false,
        ),
        (
            "09f7c1d7dfbb7df2b8fe3d3d87ee94a2259d212da4f30c1f0540d066dfa44723",
            18,
            "tia",
            "tia",
            true,
            false,
        ),
        (
            "8963217838ab4cf5cadc172203c1f0b763fbaa45f346d8ee50ba994bbcac3026",
            18,
            "ton",
            "ton",
            true,
            false,
        ),
        (
            "ddcd037c2de8dbf2a0f6eebf1c039924baf7ebf0e7eb3b44bf421af69cc1b06d",
            18,
            "trb",
            "trb",
            true,
            false,
        ),
        (
            "67aed5a24fdad045475e7195c98a98aea119c763f272d4523f5bac93a4f33c2b",
            18,
            "trx",
            "trx",
            true,
            false,
        ),
        (
            "78d185a741d07edb3412b09008b7c5cfb9bbbd7d568bf00ba737b456ba171501",
            18,
            "uni",
            "uni",
            true,
            false,
        ),
        (
            "62e158019396bf8405824b858452a1d7cc6dbb95f2e54c5641b60bb94d1f614a",
            18,
            "unibot",
            "unibot",
            true,
            false,
        ),
        (
            "d6835ad1f773de4a378115eb6824bd0c0e42d84d1c84d9750e853fb6b6c7794a",
            18,
            "wld",
            "wld",
            true,
            false,
        ),
        (
            "ec5d399846a9209f3fe5881d70aae9268c94339ff9817e8d18ff19fa05eea1c8",
            18,
            "xrp",
            "xrp",
            true,
            false,
        ),
        (
            "d183ffe0155e8a55e7274155a14ea2e8b54059cef471f88fa3f7eb4b5d8dbc24",
            18,
            "zen",
            "zen",
            true,
            false,
        ),
    ];

    tokens_config
        .into_iter()
        .map(
            |(pyth_id, decimals, name, symbol, is_shortable, is_stable)| {
                let address =
                    deploy_contract(get_contract_path("erc20"), &ctx.key_path, &ctx.endpoint);

                let pyth_id = pyth_id.to_string();
                let name = name.to_string();
                let symbol = symbol.to_string();
                let decimals = decimals as u8;

                Erc20AddressData {
                    name,
                    symbol,
                    decimals,
                    address,
                    pyth_id,
                    is_shortable,
                    is_stable,
                }
            },
        )
        .collect()
}
