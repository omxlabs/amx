use ethers::types::U256;

const INDEX_OFFSET: usize = 1000;

pub fn key_from_index(index: impl Into<U256>) -> Vec<u8> {
    let mut result = vec![0; 32];

    let index: U256 = index.into();
    let offset: U256 = INDEX_OFFSET.into();

    (index + offset).to_big_endian(&mut result);

    result
}
