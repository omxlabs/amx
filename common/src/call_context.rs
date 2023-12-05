use stylus_sdk::{call::Call, storage::TopLevelStorage};

pub trait GetCallContext {
    fn ctx(&mut self) -> Call<&mut Self>;
}

impl<T> GetCallContext for T
where
    T: TopLevelStorage,
{
    fn ctx(&mut self) -> Call<&mut Self> {
        Call::new_in(self)
    }
}
