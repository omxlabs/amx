use alloy_primitives::{I256, U256, U512};
use ruint::UintTryFrom;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MulRatioError {
    MultiplyOverflow,
    DivideByZero,
    ResultOverflow,
}

impl From<MulRatioError> for Vec<u8> {
    fn from(err: MulRatioError) -> Vec<u8> {
        use MulRatioError as E;
        let err = match err {
            E::DivideByZero => "divide by zero",
            E::MultiplyOverflow => "multiply overflow",
            E::ResultOverflow => "result overflow",
        };

        format!("MulRatio: {err}").into()
    }
}

pub fn safe_mul_ratio(a: U256, b: U256, c: U256) -> Result<U256, Vec<u8>> {
    let res = U512::from(a)
        .checked_mul(U512::from(b))
        .ok_or(MulRatioError::MultiplyOverflow)?
        .checked_div(U512::from(c))
        .ok_or(MulRatioError::DivideByZero)?;

    if res > U512::from(U256::MAX) {
        return Err(MulRatioError::ResultOverflow.into());
    }

    Ok(U256::from(res))
}

pub fn safe_mul<T>(a: U256, b: T) -> Result<U256, Vec<u8>>
where
    U256: UintTryFrom<T>,
{
    a.checked_mul(U256::from(b)).ok_or("Mul: overflow".into())
}

pub fn safe_div<T>(a: U256, b: T) -> Result<U256, Vec<u8>>
where
    U256: UintTryFrom<T>,
{
    a.checked_div(U256::from(b))
        .ok_or("Div: divide by zero".into())
}

pub fn safe_add<T>(a: U256, b: T) -> Result<U256, Vec<u8>>
where
    U256: UintTryFrom<T>,
{
    a.checked_add(U256::from(b)).ok_or("Add: overflow".into())
}

pub fn safe_sub<T>(a: U256, b: T) -> Result<U256, Vec<u8>>
where
    U256: UintTryFrom<T>,
{
    a.checked_sub(U256::from(b)).ok_or("Sub: underflow".into())
}

pub fn safe_add_int(a: U256, b: I256) -> Result<U256, Vec<u8>> {
    if b.is_negative() {
        safe_sub(a, b.unsigned_abs())
    } else {
        safe_add(a, b.unsigned_abs())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SafeSubToIntError {
    ToIntOverflow,
    NegativeOverflow,
}

impl From<SafeSubToIntError> for Vec<u8> {
    fn from(err: SafeSubToIntError) -> Vec<u8> {
        use SafeSubToIntError as E;
        let err = match err {
            E::ToIntOverflow => "to_int overflow",
            E::NegativeOverflow => "negative overflow",
        };

        format!("SafeSubToInt: {err}").into()
    }
}

pub fn safe_sub_to_int(a: U256, b: U256) -> Result<I256, Vec<u8>> {
    let dif = I256::try_from(a.abs_diff(b)).map_err(|_| SafeSubToIntError::ToIntOverflow)?;

    if a > b {
        Ok(dif)
    } else {
        dif.checked_neg()
            .ok_or(SafeSubToIntError::NegativeOverflow.into())
    }
}
