use std::time::Instant;

use pht_crypto::{Ciphertext, Plaintext};
use pht_crypto::paillier::{PartialDecryption, PublicKey};

use party::UpdateSearchRange;

use crate::party;

pub trait PartyServerTrait {
    fn add_ciphertexts(
        &mut self, lt_ciphertexts: &[Ciphertext], gt_ciphertexts: &[Ciphertext],
    ) -> [Ciphertext; 2];

    fn combine_shares(
        &self, lt_shares: &[PartialDecryption], gt_shares: &[PartialDecryption],
    ) -> [Plaintext; 2];

    fn calculate_update(&self, plaintexts: [Plaintext; 2]) -> UpdateSearchRange;
}

pub struct PartyServer {
    // number of parties
    pub(crate) n: usize,
    // (a, b) range of elements in union of database
    pub(crate) pk: PublicKey,
    // k-th element to retrieve
    pub(crate) k: usize,
    // sum of the sizes of the databases
    pub(crate) databases_size: usize,
}

impl PartyServerTrait for PartyServer {
    fn add_ciphertexts(
        &mut self,
        lt_ciphertexts: &[Ciphertext],
        gt_ciphertexts: &[Ciphertext],
    ) -> [Ciphertext; 2] {

        let now = Instant::now();
        let add_fn = | ciphertexts: & [Ciphertext] | {
            ciphertexts
                .iter()
                .fold(Ciphertext::from(1), | mut acc, cipher| {
                    self.pk.add_encrypted( & mut acc, cipher);
                    acc
                })
        };
        let (lt_share, gt_share) = rayon::join(
            || add_fn(lt_ciphertexts),
            || add_fn(gt_ciphertexts),
        );
        tracing::trace!(elapsed_ms = % now.elapsed().as_millis(), "Add ciphertexts w/o overhead");
        [lt_share, gt_share]
    }

    fn combine_shares(
        &self,
        lt_shares: &[PartialDecryption],
        gt_shares: &[PartialDecryption],
    ) -> [Plaintext; 2] {
        let (lt, gt) = rayon::join(
            || self.pk.share_combine(lt_shares).unwrap(),
            || self.pk.share_combine(gt_shares).unwrap(),
        );
        [lt, gt]
    }

    fn calculate_update(&self, [lt, gt]: [Plaintext; 2]) -> UpdateSearchRange {
        if lt > self.databases_size || gt > self.databases_size { // TODO: Fix this error and remove the case
            UpdateSearchRange::Abort
        } else if lt >= self.k {
            UpdateSearchRange::SearchBelow
        } else if gt > self.databases_size - self.k {
            UpdateSearchRange::SearchAbove
        } else {
            UpdateSearchRange::FoundK
        }
    }
}