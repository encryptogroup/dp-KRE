use std::fmt;

use pht_crypto::paillier::{generate_key_pair, Polynomial};
use rand::{Rng, thread_rng};
use rand::distributions::Distribution;
use rand::distributions::uniform::SampleUniform;
use rand_distr::Normal;
use rug::rand::RandState;

use crate::party::dp_client::{DPClient, GetScaleFn, NoiseLevel};
use crate::party::party_client::PartyClient;
use crate::party::party_server::PartyServer;
use crate::party::TypeTrait;

// This file contains functions that are useful for testing the protocol.

/// Represents the K value used for the protocol.
#[derive(Copy, Clone)]
pub enum KValue {
    Min,
    Median,
    Max,
}

impl KValue {
    pub fn to_k(&self, n: usize) -> usize {
        match self {
            KValue::Min => 1,
            KValue::Median => n / 2,
            KValue::Max => n,
        }
    }
}

impl fmt::Display for KValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KValue::Min => write!(f, "min"),
            KValue::Median => write!(f, "median"),
            KValue::Max => write!(f, "max"),
        }
    }
}


/// Creates a server and a single party client with the given database.
pub(crate) fn create_single_party(db: Vec<i32>) -> (PartyServer, PartyClient<i32>) {
    let idx = 0;
    let n = 1;
    let k = 1;
    let databases_size = db.len();
    let min = db.iter().min().unwrap();
    let max = db.iter().max().unwrap();
    let search_range = [*min, *max];
    let mut rand = RandState::new();
    let (pk, sk) = generate_key_pair(128, 1, 1).unwrap();
    let key_share = Polynomial::new(&sk, &mut rand).compute(idx);
    let client = PartyClient::new(
        db,
        idx,
        n,
        k,
        databases_size,
        search_range,
        pk.clone(),
        key_share,
        rand,
    );
    let server = PartyServer {
        n,
        pk,
        k,
        databases_size,
    };
    (server, client)
}

/// Creates a server and multiple party clients with the given vector of database.
///
/// The number of clients is equal to the length of the databases.
pub fn create_server_clients(k: usize, databases: Vec<Vec<i32>>) -> (PartyServer, Vec<PartyClient<i32>>) {
    let n = databases.len();
    let databases_size = databases.iter().map(|db| db.len()).sum();
    let min = databases.iter().map(|db| db.iter().min().unwrap()).min().unwrap();
    let max = databases.iter().map(|db| db.iter().max().unwrap()).max().unwrap();
    let search_range = [*min, *max];
    let mut rand = RandState::new();
    let (pk, sk) = generate_key_pair(128, n as u32, n as u32).unwrap();
    let poly = Polynomial::new(&sk, &mut rand);
    let clients = databases
        .into_iter()
        .enumerate()
        .map(|(idx, db)| {
            let idx = idx as u32;
            let key_share = poly.compute(idx);
            let rand = RandState::new();
            PartyClient::new(
                db,
                idx,
                n,
                k,
                databases_size,
                search_range.clone(),
                pk.clone(),
                key_share,
                rand.clone(),
            )
        })
        .collect();
    let server = PartyServer {
        n,
        pk,
        k,
        databases_size,
    };
    (server, clients)
}

/// Creates a server and multiple party clients that use differential privacy.
pub fn create_server_dp_clients(k: usize, databases: Vec<Vec<i32>>, get_scale_fn: GetScaleFn,
                                noise_level: NoiseLevel) -> (PartyServer, Vec<DPClient<i32>>) {
    let (server, clients) = create_server_clients(k, databases);
    let dp_clients = clients.into_iter().map(|client| -> DPClient<i32> {
        DPClient::new(client, get_scale_fn, noise_level)
    }).collect();
    (server, dp_clients)
}

/// Samples a vector of databases with random elements.
///
/// # Arguments
/// * `db_size` - The total size of the union of all individual databases.
/// * `num_parties` - The number of parties.
/// * `min` - The minimum value allowed to be in the databases.
/// * `max` - The maximum value allowed to be in the databases.
/// # Example
/// ```
/// use privdev_dp_comp::utils::protocol::sample_databases;
/// let dbs = sample_databases(10, 3, -10, 10);
/// println!("{:?}", dbs); // Possible result: [-4, 3, 10, 5], [9, 4, 0], [5, -7, -1]
/// ```
///
pub fn sample_databases<T: TypeTrait + SampleUniform>(db_size: usize, num_parties: usize, min: T, max: T) -> Vec<Vec<T>> {
    //let db_sizes = sample_db_sizes(db_size, num_parties);
    let db_sizes = get_db_sizes(db_size, num_parties);
    let mut dbs = Vec::with_capacity(db_size);
    for i in 0..num_parties {
        dbs.push(sample_database::<T>(db_sizes[i], min.clone(), max.clone()));
    }
    dbs
}

pub fn sample_database<T: TypeTrait + SampleUniform>(size: usize, min: T, max: T) -> Vec<T> {
    let mut rnd = thread_rng();
    let mut db = Vec::with_capacity(size);
    for _ in 0..size {
        db.push(rnd.gen_range(min.clone()..max.clone()));
    }
    db
}

pub(crate) fn get_db_sizes(total_size: usize, num_parties: usize) -> Vec<usize> {
    let size_per_party = total_size / num_parties;
    vec![size_per_party; num_parties]
}

/// Samples a vector of database sizes for num_parties that sum up to `total_size`.
/// It is ensured that each party has at least a database of size 1.
///
/// Example: When running `sample_db_sizes(100, 3)`, a possible result could be
/// `[30, 30, 40]`, `[11, 65, 24]`, or `[90, 9, 1]` and so on.

pub(crate) fn sample_db_sizes(total_size: usize, num_parties: usize) -> Vec<usize> {
    if total_size < num_parties {
        panic!("Total size must be greater than number of parties");
    }

    let mut rnd = thread_rng();
    let average = total_size / num_parties;
    let deviation = average / 4;
    let distribution = Normal::new(average as f64, deviation as f64).unwrap();

    let mut sizes = vec![1; num_parties];

    let mut remaining = total_size - num_parties;
    for i in 0..num_parties {
        let size = distribution.sample(&mut rnd).min(remaining as f64);
        sizes[i] += size as usize;
        remaining -= size as usize
    }
    if remaining > 0 {
        let idx = rnd.gen_range(0..num_parties);
        sizes[idx] += remaining;
    }
    sizes
}

/// Returns the k-th element of the union of all databases.
pub(crate) fn get_kth_element<T: TypeTrait>(db: &Vec<Vec<T>>, k: usize) -> T {
    let mut db = db.iter().flatten().collect::<Vec<&T>>();
    db.sort();
    db[k - 1].clone()
}
