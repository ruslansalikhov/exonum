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

//! Cryptocurrency database schema.

use exonum::{
    crypto,
    crypto::{Hash, PublicKey},
    storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot},
};

use crate::{wallet::Wallet, transferproposal::TransferProposal, INITIAL_BALANCE};

/// Database schema for the cryptocurrency.
#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> AsMut<T> for Schema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Creates a new schema from the database view.
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    /// Returns `ProofMapIndex` with wallets.
    pub fn wallets(&self) -> ProofMapIndex<&T, Hash, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &self.view)
    }

    /// Returns history of the wallet with the given user name hash.
    pub fn wallet_history(&self, hash: &Hash) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", hash, &self.view)
    }

    /// Returns wallet for the username hash.
    pub fn wallet(&self, hash: &Hash) -> Option<Wallet> {
        self.wallets().get(hash)
    }

    /// Returns `ProofMapIndex` with wallets.
    pub fn transfer_proposals(&self) -> ProofMapIndex<&T, Hash, TransferProposal> {
        ProofMapIndex::new("cryptocurrency.transfer_proposals", &self.view)
    }

    /// Returns proposals for the given tranfer hash.
    pub fn transfer_proposal(&self, hash: &Hash) -> Option<TransferProposal> {
        self.transfer_proposals().get(hash)
    }

    /// Returns the state hash of cryptocurrency service.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.wallets().merkle_root()]
    }
}

/// Implementation of mutable methods.
impl<'a> Schema<&'a mut Fork> {
    /// Returns mutable `ProofMapIndex` with wallets.
    pub fn wallets_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &mut self.view)
    }

    /// Returns mutable `ProofMapIndex` with transfer proposals.
    pub fn transfer_proposals_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, TransferProposal> {
        ProofMapIndex::new("cryptocurrency.transfer_proposals", &mut self.view)
    }

    /// Returns history for the wallet by the given public key.
    pub fn wallet_history_mut(
        &mut self,
        hash: &Hash,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", hash, &mut self.view)
    }

    /// Increase balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn increase_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(&crypto::hash(wallet.name.as_bytes()));
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance;
            wallet.set_balance(balance + amount, &history_hash)
        };
        self.wallets_mut().put(&crypto::hash(wallet.name.as_bytes()), wallet.clone());
    }

    /// Decrease balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn decrease_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(&crypto::hash(wallet.name.as_bytes()));
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance;
            wallet.set_balance(balance - amount, &history_hash)
        };
        self.wallets_mut().put(&crypto::hash(wallet.name.as_bytes()), wallet.clone());
    }

    /// Create new wallet and append first record to its history.
    pub fn create_wallet(&mut self, name: &String, keys: &Vec<PublicKey>, quorum: u32, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(&crypto::hash(name.as_bytes()));
            history.push(*transaction);
            let history_hash = history.merkle_root();
            Wallet::new(name, keys.to_vec(), quorum, INITIAL_BALANCE, history.len(), &history_hash)
        };
        self.wallets_mut().put(&crypto::hash(name.as_bytes()), wallet);
    }

    /// Create new transfer proposal.
    pub fn create_transfer_proposal(&mut self, from: &String, to: &String, amount: u64, seed: u64) {
        let transfer_proposal = TransferProposal::new(from, to, amount, seed, 1);
        self.transfer_proposals_mut().put(&transfer_proposal.hash(), transfer_proposal);
    }

    /// Increase proposal signs
    pub fn increase_proposal_signs(&mut self, transfer_proposal: TransferProposal) {
        let transfer_proposal = {
            transfer_proposal.increase_signs()
        };
        self.transfer_proposals_mut().put(&transfer_proposal.hash(), transfer_proposal.clone());
    }
}
