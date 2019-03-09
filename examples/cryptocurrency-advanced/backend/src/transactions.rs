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

//! Cryptocurrency transactions.

// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto,
    crypto::{PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};

use super::proto;
use crate::{schema::Schema, CRYPTOCURRENCY_SERVICE_ID};
use crate::transferproposal::TransferProposal;

const ERROR_SENDER_SAME_AS_RECEIVER: u8 = 0;
const ERROR_SENDER_WRONG_KEY: u8 = 1;

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    /// Not enough signs yet.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Not enough signs yet")]
    NotEnoughSignsYet = 4,

    /// Quorum number is not correct.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Quorum is not correct")]
    QuorumIsNotCorrect = 5,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

/// Transfer `amount` of the currency from one wallet to another.
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Transfer", serde_pb_convert)]
pub struct Transfer {
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
}

/// Issue `amount` of the currency to the `wallet`.
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Issue")]
pub struct Issue {
    /// Recipient.
    pub to: String,
    /// Issued amount of currency.
    pub amount: u64,
    /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
    ///
    /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
    pub seed: u64,
}

/// Create wallet with the given `name`.
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::CreateWallet")]
pub struct CreateWallet {
    /// Name of the new wallet.
    pub name: String,
    /// Wallet keys.
    pub pub_keys: Vec<PublicKey>,
    /// Quorum size.
    pub quorum: u32,
}

/// Transaction group.
#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum WalletTransactions {
    /// Transfer tx.
    Transfer(Transfer),
    /// Issue tx.
    Issue(Issue),
    /// CreateWallet tx.
    CreateWallet(CreateWallet),
}

impl CreateWallet {
    #[doc(hidden)]
    pub fn sign(name: &str, pub_keys: Vec<PublicKey>, quorum: u32, pk: &PublicKey, sk: &SecretKey) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self {
                name: name.to_owned(),
                pub_keys,
                quorum
            },
            CRYPTOCURRENCY_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Issue {
    #[doc(hidden)]
    pub fn sign(
        to: String,
        amount: u64,
        seed: u64,
        pk: &PublicKey,
        sk: &SecretKey,
    ) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self { to, amount, seed },
            CRYPTOCURRENCY_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Transfer {
    #[doc(hidden)]
    pub fn sign(
        from: String,
        to: String,
        amount: u64,
        seed: u64,
        pk: &PublicKey,
        sk: &SecretKey,
    ) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self { from, to, amount, seed },
            CRYPTOCURRENCY_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Transaction for Transfer {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let from = &self.from;
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let to = &self.to;
        let amount = self.amount;
        let seed = self.seed;

        if from == to {
            return Err(ExecutionError::new(ERROR_SENDER_SAME_AS_RECEIVER));
        }

        let sender = schema.wallet(&crypto::hash(from.as_bytes())).ok_or(Error::SenderNotFound)?;

        let mut ok = false;
        for key in &sender.pub_keys {
            if pub_key.to_hex() == key.to_hex() {
                ok = true;
            }
        }

        if !ok {
            return Err(ExecutionError::new(ERROR_SENDER_WRONG_KEY));
        }

        let receiver = schema.wallet(&crypto::hash(to.as_bytes())).ok_or(Error::ReceiverNotFound)?;

        let transfer_proposal = TransferProposal::new(from, to, amount, seed, 1);
        if schema.transfer_proposal(&transfer_proposal.hash()).is_none() {
            schema.create_transfer_proposal(from, to, amount, seed);
        } else {
            schema.increase_proposal_signs(transfer_proposal);
        }
        let transfer_proposal = schema.transfer_proposal(
            &TransferProposal::new(from, to, amount, seed, 1).hash()).unwrap();

        if transfer_proposal.signs != sender.quorum {
            //Err(Error::NotEnoughSignsYet)?
            Ok(())
        } else {
            if sender.balance < amount {
                Err(Error::InsufficientCurrencyAmount)?
            }

            schema.decrease_wallet_balance(sender, amount, &hash);
            schema.increase_wallet_balance(receiver, amount, &hash);

            Ok(())
        }
    }
}

impl Transaction for Issue {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let _pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if let Some(wallet) = schema.wallet(&crypto::hash(self.to.as_bytes())) {
            schema.increase_wallet_balance(wallet, self.amount, &hash);
            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }
    }
}

impl Transaction for CreateWallet {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if pub_key.to_hex() != self.pub_keys[0].to_hex() {
            return Err(ExecutionError::new(ERROR_SENDER_WRONG_KEY));
        }

        let name = &self.name;
        if schema.wallet(&crypto::hash(name.as_bytes())).is_none() {

            if self.quorum == 0 || self.quorum > self.pub_keys.len() as u32 {
                Err(Error::QuorumIsNotCorrect)?
            }

            schema.create_wallet(name, &self.pub_keys, self.quorum, &hash);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        }
    }
}
