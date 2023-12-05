extern crate alloc;

use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    event Swap(address account, address token_in, address token_out, uint256 amount_in, uint256 amount_out);
}

sol_interface! {
    interface ISwapRouter {
        function init(address weth, address usdo, address vault, address swap_manager, address positions_router) external;

        function directPoolDeposit(address token, uint256 amount) external;

        function swapForPosition(address[] memory path, uint256 min_out, address receiver) external returns (uint256);

        function swap(address[] memory path, uint256 amount_in, uint256 min_out, address receiver) external;

        function swapEthToTokens(address[] memory path, uint256 min_out, address receiver) external payable;

        function swapToEth(address token_in, uint256 amount_in, uint256 min_out, address receiver) external;
    }
}

sol_interface! {
    interface IPositionsIncreaseRouter {
        function init(address weth, address vault, address positions_increase_manager, address swap_router) external;

        function increasePosition(address[] memory path, address index_token, uint256 amount_in, uint256 min_out, uint256 size_delta, bool is_long, uint256 price) external;

        function increasePositionEth(address[] memory path, address index_token, uint256 min_out, uint256 size_delta, bool is_long, uint256 price) external payable;
    }
}

sol_interface! {
    interface IPositionsDecreaseRouter {
        function init(address weth, address vault, address positions_decrease_manager, address swap_router) external;

        function decreasePosition(address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long, address receiver, uint256 price) external returns (uint256);

        function decreasePositionEth(address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long, address receiver, uint256 price) external returns (uint256);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RouterError {
    Forbidden,
    AlreadyInitialized,
    HighMarkPrice,
    LowMarkPrice,
    InvalidPlugin,
    PluginNotApproved,
    InvalidPath,
    InvalidTokenIn,
    InvalidTokenOut,
    InsufficientAmountOut,
}

impl From<RouterError> for Vec<u8> {
    fn from(err: RouterError) -> Vec<u8> {
        use RouterError as E;
        let err = match err {
            E::Forbidden => "forbidden",
            E::AlreadyInitialized => "already initialized",
            E::HighMarkPrice => "mark price higher than limit",
            E::LowMarkPrice => "mark price lower than limit",
            E::InvalidPlugin => "invalid plugin",
            E::PluginNotApproved => "plugin not approved",
            E::InvalidPath => "invalid path",
            E::InvalidTokenIn => "invalid token in",
            E::InvalidTokenOut => "invalid token out",
            E::InsufficientAmountOut => "insufficient amount out",
        };

        format!("Router: {err}").into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SwapPath {
    /// transfer from token_in to token_out directly
    Direct {
        token_in: Address,
        token_out: Address,
    },
    /// transfer from token_in to token_out via an intermediate token
    Indirect {
        token_in: Address,
        token_out: Address,
        intermediate: Address,
    },
}

impl SwapPath {
    #[inline]
    pub fn from_arr(path: Vec<Address>) -> Result<Self, Vec<u8>> {
        if path.len() == 2 {
            Ok(SwapPath::Direct {
                token_in: path[0],
                token_out: path[1],
            })
        } else if path.len() == 3 {
            Ok(SwapPath::Indirect {
                token_in: path[0],
                intermediate: path[1],
                token_out: path[2],
            })
        } else {
            Err(RouterError::InvalidPath.into())
        }
    }

    pub fn to_vec(&self) -> Vec<Address> {
        match self {
            SwapPath::Direct {
                token_in,
                token_out,
            } => vec![*token_in, *token_out],
            SwapPath::Indirect {
                token_in,
                intermediate,
                token_out,
            } => vec![*token_in, *intermediate, *token_out],
        }
    }

    pub fn token_in(&self) -> Address {
        match self {
            SwapPath::Direct { token_in, .. } => *token_in,
            SwapPath::Indirect { token_in, .. } => *token_in,
        }
    }

    pub fn token_out(&self) -> Address {
        match self {
            SwapPath::Direct { token_out, .. } => *token_out,
            SwapPath::Indirect { token_out, .. } => *token_out,
        }
    }

    pub fn second_token(&self) -> Address {
        match self {
            SwapPath::Direct { token_out, .. } => *token_out,
            SwapPath::Indirect { intermediate, .. } => *intermediate,
        }
    }

    pub fn intermediate(&self) -> Option<Address> {
        match self {
            SwapPath::Direct { .. } => None,
            SwapPath::Indirect { intermediate, .. } => Some(*intermediate),
        }
    }

    pub fn is_indirect(&self) -> bool {
        matches!(self, SwapPath::Indirect { .. })
    }

    pub fn is_direct(&self) -> bool {
        matches!(self, SwapPath::Direct { .. })
    }

    pub fn unwrap_direct(&self) -> (Address, Address) {
        match self {
            SwapPath::Direct {
                token_in,
                token_out,
            } => (*token_in, *token_out),
            SwapPath::Indirect { .. } => panic!("unwrap_direct called on indirect path"),
        }
    }

    pub fn unwrap_indirect(&self) -> (Address, Address, Address) {
        match self {
            SwapPath::Direct { .. } => panic!("unwrap_indirect called on direct path"),
            SwapPath::Indirect {
                token_in,
                token_out,
                intermediate,
            } => (*token_in, *intermediate, *token_out),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CollateralPath {
    Token(Address),
    Path(SwapPath),
}

impl CollateralPath {
    #[inline]
    pub fn from_arr(path: Vec<Address>) -> Result<Self, Vec<u8>> {
        if path.len() == 1 {
            Ok(CollateralPath::Token(path[0]))
        } else {
            Ok(CollateralPath::Path(SwapPath::from_arr(path)?))
        }
    }

    pub fn is_token(&self) -> bool {
        matches!(self, CollateralPath::Token(_))
    }

    pub fn is_path(&self) -> bool {
        matches!(self, CollateralPath::Path(_))
    }

    pub fn unwrap_token(&self) -> Address {
        match self {
            CollateralPath::Token(token) => *token,
            CollateralPath::Path(_) => panic!("unwrap_token called on path"),
        }
    }

    pub fn unwrap_path(&self) -> SwapPath {
        match self {
            CollateralPath::Token(_) => panic!("unwrap_path called on token"),
            CollateralPath::Path(path) => *path,
        }
    }

    pub fn is_direct(&self) -> bool {
        match self {
            CollateralPath::Token(_) => false,
            CollateralPath::Path(path) => path.is_direct(),
        }
    }

    pub fn is_indirect(&self) -> bool {
        match self {
            CollateralPath::Token(_) => false,
            CollateralPath::Path(path) => path.is_indirect(),
        }
    }

    pub fn unwrap_direct(&self) -> (Address, Address) {
        match self {
            CollateralPath::Token(_) => panic!("unwrap_direct called on token"),
            CollateralPath::Path(path) => path.unwrap_direct(),
        }
    }

    pub fn unwrap_indirect(&self) -> (Address, Address, Address) {
        match self {
            CollateralPath::Token(_) => panic!("unwrap_indirect called on token"),
            CollateralPath::Path(path) => path.unwrap_indirect(),
        }
    }

    pub fn token_in(&self) -> Address {
        match self {
            CollateralPath::Token(token) => *token,
            CollateralPath::Path(path) => path.token_in(),
        }
    }

    pub fn token_out(&self) -> Address {
        match self {
            CollateralPath::Token(token) => *token,
            CollateralPath::Path(path) => path.token_out(),
        }
    }
}
