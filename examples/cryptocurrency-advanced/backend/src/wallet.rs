// Copyright 2019 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Cryptocurrency wallet.

use exonum::crypto::{Hash, PublicKey};

use super::proto;

/// Wallet information stored in the database.
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Wallet", serde_pb_convert)]
pub struct Wallet {
    /// Name of the wallet.
    pub name: String,
    /// `PublicKey` of the wallet.
    pub pub_keys: Vec<PublicKey>,
    /// Quorum number.
    pub quorum: u32,
    /// Current balance of the wallet.
    pub balance: u64,
    /// Length of the transactions history.
    pub history_len: u64,
    /// `Hash` of the transactions history.
    pub history_hash: Hash,
}

impl Wallet {
    /// Create new Wallet.
    pub fn new(
        name: &str,
        pub_keys: Vec<PublicKey>,
        quorum: u32,
        balance: u64,
        history_len: u64,
        &history_hash: &Hash,
    ) -> Self {
        Self {
            name: name.to_owned(),
            pub_keys,
            quorum,
            balance,
            history_len,
            history_hash,
        }
    }
    /// Returns a copy of this wallet with updated balance.
    pub fn set_balance(self, balance: u64, history_hash: &Hash) -> Self {
        Self::new(
            &self.name,
            self.pub_keys,
            self.quorum,
            balance,
            self.history_len + 1,
            history_hash,
        )
    }
}
