use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;
use std::time::Instant;

use num::one;
use pht_crypto::Ciphertext;
use pht_crypto::paillier::{PartialDecryption, PrivateKeyShare, PublicKey};
use rug::rand::RandState;

use crate::party::*;

pub trait PartyClientTrait<T>: Send where T: TypeTrait {
    fn local_computation(&mut self) -> [Ciphertext; 2];

    fn compute_shares(&self, lt: Ciphertext, gt: Ciphertext) -> [PartialDecryption; 2];

    fn update_search_range(&mut self, update: UpdateSearchRange) -> Option<T>;
}

#[derive(Clone, Debug)]
pub struct PartyClient<T> where T: TypeTrait {
    // database of element
    pub(crate) database: Arc<Vec<T>>,
    // index of the party_old
    pub(crate) idx: u32,
    // number of parties
    pub(crate) n: usize,
    // k-th element to retrieve
    pub(crate) k: usize,
    // sum of the sizes of the databases
    pub(crate) databases_size: usize,
    // (a, b) range of elements in union of database
    pub(crate) search_range: [T; 2],
    // Search range idx is an optimization that uses the fact that we know the index at which the
    // elements are bigger than m. This is used to slice the database before the local computation.
    // However, this optimization does only work for min/max queries.
    pub(crate) search_range_idx: Range<usize>,
    pub(crate) greater_than_m_idx: Option<usize>,
    // middle-point
    pub(crate) m: T,
    pub(crate) pk: PublicKey,
    pub(crate) key_share: PrivateKeyShare,
    pub(crate) rand: RandState<'static>,
}

unsafe impl<T> Send for PartyClient<T>
    where
        T: TypeTrait,
{
}

impl<T> PartyClientTrait<T> for PartyClient<T> where T: TypeTrait {

    fn local_computation(&mut self) -> [Ciphertext; 2] {
        let now = Instant::now();
        let [less, greater] = self.local_comp1();
        tracing::trace!(elapsed_ms = %now.elapsed().as_millis(), "local computation comparisons");

        let now = Instant::now();
        let [lt, gt] = self.local_comp2(less, greater);
        tracing::trace!(elapsed_ms = %now.elapsed().as_millis(), "local computation encryption");
        [lt, gt]
    }

    fn compute_shares(&self, lt: Ciphertext, gt: Ciphertext) -> [PartialDecryption; 2] {
        let now = Instant::now();
        let (lt, gt) = rayon::join(
            || self.key_share.share_decrypt(&self.pk, lt),
            || self.key_share.share_decrypt(&self.pk, gt),
        );
        tracing::trace!(elapsed_ms = %now.elapsed().as_millis(), "compute shares");
        [lt, gt]
    }

    fn update_search_range(&mut self, update: UpdateSearchRange) -> Option<T> {
        match update {
            UpdateSearchRange::FoundK => Some(self.m.clone()),
            UpdateSearchRange::SearchBelow => {
                self.search_range[1] = self.m.clone() - one();
                if let Some(idx) = self.greater_than_m_idx {
                    self.search_range_idx.end = idx;
                }
                None
            }
            UpdateSearchRange::SearchAbove => {
                self.search_range[0] = self.m.clone() + one();
                if let Some(idx) = self.greater_than_m_idx {
                    self.search_range_idx.start = idx;
                }
                None
            }
            _ => { None }
        }
    }
}

impl<T> PartyClient<T> where T: TypeTrait {
    pub fn new(
        mut database: Vec<T>,
        idx: u32,
        n: usize,
        k: usize,
        databases_size: usize,
        search_range: [T; 2],
        pk: PublicKey,
        key_share: PrivateKeyShare,
        rand: RandState<'static>,
    ) -> Self {
        database.sort();
        Self {
            search_range_idx: 0..database.len(),
            database: Arc::new(database),
            idx,
            n,
            k,
            databases_size,
            m: search_range[0].average_floor(&search_range[1]),
            search_range,
            pk,
            key_share,
            rand,
            greater_than_m_idx: None,
        }
    }

    pub(crate) fn local_comp1(&mut self) -> [usize; 2] {
        self.m = self.search_range[0].average_floor(&self.search_range[1]);
        // If we are searching for the min/max, we can slice the database to a shorter search range.
        let range_idx = if self.k == 1 || self.k == self.databases_size { self.search_range_idx.clone() } else { 0..self.database.len() };
        let ([less, greater], greater_than_m_idx) = self.database[range_idx]
            .iter()
            .enumerate()
            .fold(
                ([0, 0], None),
                |([mut less, mut greater], mut greater_than_m_idx), (idx, el)| {
                    match el.cmp(&self.m) {
                        Ordering::Less => {
                            less += 1;
                        }
                        Ordering::Greater => {
                            greater += 1;
                            greater_than_m_idx.get_or_insert(idx);
                        }
                        Ordering::Equal => (),
                    };
                    ([less, greater], greater_than_m_idx)
                },
            );

        // Add search_range_idx.start because we sliced the database before the enumerate.
        self.greater_than_m_idx = greater_than_m_idx.map(|idx| idx + self.search_range_idx.start);
        [less, greater]
    }

    pub(crate) fn local_comp2(&mut self, less: usize, greater: usize) -> [Ciphertext; 2] {
        let now = Instant::now();
        let less_enc = self.pk.encrypt(less.into(), &mut self.rand);
        let greater_enc = self.pk.encrypt(greater.into(), &mut self.rand);
        tracing::trace!(elapsed_ms = %now.elapsed().as_millis(), "Local computation Enc");
        [less_enc, greater_enc]
    }

    pub fn get_pk(&self) -> &PublicKey {
        &self.pk
    }

    pub fn get_key_share(&self) -> &PrivateKeyShare {
        &self.key_share
    }

    pub fn get_databases_size(&self) -> usize {
        self.databases_size
    }
}