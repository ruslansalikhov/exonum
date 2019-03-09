//! Transfer proposal

use exonum::crypto::{Hash, HashStream};

use super::proto;

/// Wallet information stored in the database.
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::TransferProposal", serde_pb_convert)]
pub struct TransferProposal {
    /// name of receiver's wallet.
    pub from: String,
    /// name of receiver's wallet.
    pub to: String,
    /// Amount of currency to transfer.
    pub amount: u64,
    /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
    ///
    /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
    pub seed: u64,
    /// Number of signs
    pub signs: u32
}

impl TransferProposal {
    /// Create new Wallet.
    pub fn new(
        from: &str,
        to: &str,
        amount: u64,
        seed: u64,
        signs: u32,
    ) -> Self {
        Self {
            from: from.to_owned(),
            to: to.to_owned(),
            amount,
            seed,
            signs,
        }
    }
    /// Returns a copy of this wallet with updated balance.
    pub fn increase_signs(self) -> Self {
        Self::new(
            &self.from,
            &self.to,
            self.amount,
            self.seed,
            self.signs + 1
        )
    }
    /// Returns hash
    pub fn hash(&self) -> Hash {
        return HashStream::new()
            .update(&self.from.as_bytes())
            .update(&self.to.as_bytes())
            .update(&self.amount.to_le_bytes())
            .update(&self.seed.to_le_bytes())
            .hash();
    }
}
