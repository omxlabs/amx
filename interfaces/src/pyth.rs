extern crate alloc;

use alloy_primitives::{Address, FixedBytes};
use alloy_sol_types::{sol, SolType};
use stylus_sdk::{
    abi::{AbiType, ConstString, FixedBytesSolType},
    call,
};

sol! {
    /// A price with a degree of uncertainty, represented as a price +- a confidence interval.
    ///
    /// The confidence interval roughly corresponds to the standard error of a normal distribution.
    /// Both the price and confidence are stored in a fixed-point numeric representation,
    /// `x * (10^expo)`, where `expo` is the exponent.
    ///
    /// Please refer to the documentation at https://docs.pyth.network/consumers/best-practices for how
    /// to how this price safely.
    struct Price {
        /// Price
        int64 price;
        /// Confidence interval around the price
        uint64 conf;
        /// Price exponent
        int32 expo;
        /// Unix timestamp describing when the price was published
        uint publish_time;
        bytes32 id;
    }
}

impl AbiType for Price {
    type SolType = Self;
    const ABI: ConstString = ConstString::new("Price");
}

pub struct IPyth {
    pub address: Address,
}

/// NOTE: `sol_interface!` macro does not support custom return types yet (such as `Price`)
/// so we have to implement the interface manually.
impl IPyth {
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// `function getPrice(bytes32 id) external view returns (PythStructs.Price memory price);`
    pub fn get_price(
        &self,
        context: impl call::StaticCallContext,
        id: FixedBytes<32>,
    ) -> Result<Price, call::Error> {
        use alloc::vec;

        let args = <(FixedBytesSolType<32>,) as SolType>::encode(&(id,));
        let mut calldata = vec![209u8, 120u8, 67u8, 197u8];
        calldata.extend(args);
        let returned = call::static_call(context, self.address, &calldata)?;
        Ok(<Price as stylus_sdk::alloy_sol_types::SolType>::decode(
            &returned, true,
        )?)
    }
}
