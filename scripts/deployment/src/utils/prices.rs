use ethers::types::U256;

pub fn to_chainlink_price<T>(value: T) -> U256
where
    U256: From<T>,
{
    let base = U256::from_dec_str("1000000000000000000000000000000").unwrap();
    U256::from(value) * base
}

pub fn expand_decimals<T>(value: T, decimals: u8) -> U256
where
    U256: From<T>,
{
    let base = U256::from_dec_str("10").unwrap();
    U256::from(value) * base.pow(decimals.into())
}
