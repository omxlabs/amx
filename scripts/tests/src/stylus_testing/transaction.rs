use std::fmt::Display;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TransactionKey(pub u64);

impl TransactionKey {
    pub fn next(mut self) -> Self {
        let key = self.0;
        self.0 += 1;
        Self(key)
    }
}

impl Display for TransactionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tx_{}", self.0)
    }
}

impl From<u64> for TransactionKey {
    fn from(key: u64) -> Self {
        Self(key)
    }
}

impl From<TransactionKey> for u64 {
    fn from(key: TransactionKey) -> Self {
        key.0
    }
}

impl std::ops::Deref for TransactionKey {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TransactionKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
