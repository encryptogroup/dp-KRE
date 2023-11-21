use core::fmt;
use std::fmt::Formatter;
use std::time::Instant;
use pht_crypto::Ciphertext;
use pht_crypto::paillier::PartialDecryption;
use rand::{Rng};
use rand::distributions::Uniform;
use rand_distr::Distribution;
use crate::party::party_client::{PartyClient, PartyClientTrait};
use crate::party::{TypeTrait, UpdateSearchRange};

#[derive(PartialEq, Copy, Clone)]
pub enum NoiseLevel {
    NONE,
    LOW,
    MEDIUM,
    HIGH,
}

impl fmt::Display for NoiseLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NoiseLevel::NONE => write!(f, "none"),
            NoiseLevel::LOW => write!(f, "low"),
            NoiseLevel::MEDIUM => write!(f, "medium"),
            NoiseLevel::HIGH => write!(f, "high"),
        }
    }
}

/// Computes the scale for the laplace distribution given the result and the database size.
pub type GetScaleFn = fn(NoiseLevel, usize, usize) -> f64;

/// An exemplary function for computing the scale as a fixed value given the noise level.
pub fn get_scale_fixed(noise_level: NoiseLevel, _: usize, _: usize) -> f64 {
    match noise_level {
        NoiseLevel::NONE => 0.0,
        NoiseLevel::LOW => 0.2,
        NoiseLevel::MEDIUM => 0.5,
        NoiseLevel::HIGH => 2.0,
    }
}

/// A function for computing the scale as a sigmoid function given result/db_size.
pub fn get_scale_sigmoid(noise_level: NoiseLevel, result: usize, db_size: usize) -> f64 {
    let db_size = db_size as f64;
    // The logarithm of the db size is used as the scale for the sigmoid function. Intuitively, it
    // represents the maximum value it will return (given result=db_size).
    let db_size_log = db_size.log(100.0);
    // Compute the compression and scale of the sigmoid function s.t. the scale is dependent on the
    // number of elements in the database. With compression = 10 and scale = 1, the function is
    // approximately 1 at ratio = 1.0.
    let (compression, scale_sigmoid) = match noise_level {
        NoiseLevel::NONE => (0.0, 0.0),
        NoiseLevel::LOW => (5.0, db_size_log),
        NoiseLevel::MEDIUM => (10.0, 1.5 * db_size_log),
        NoiseLevel::HIGH => (15.0, 2.0 * db_size_log),
    };

    // Compute f(ratio) = 1 / (1+ e^(-ratio * factor + 5.0) * limit
    let ratio = result as f64 / db_size;
    let e = (-ratio * compression + 5.0).exp();
    let scale = 1.0 / (1.0 + e) * scale_sigmoid;
    scale
}

pub struct DPClient<T> where T: TypeTrait {
    client: PartyClient<T>,
    noise_level: NoiseLevel,
    get_scale_fn: GetScaleFn,
    pub noise_array: Vec<f64>,
}

impl<T> PartyClientTrait<T> for DPClient<T> where T: TypeTrait {
    fn local_computation(&mut self) -> [Ciphertext; 2] {
        let now = Instant::now();
        let [mut less, mut greater] = self.client.local_comp1();
        tracing::trace!(elapsed_ms = %now.elapsed().as_millis(), "local computation comparisons");

        // Add laplace noise to the less and greater counts.
        less = self.add_noise(less);
        greater = self.add_noise(greater);

        // If the sum of the new less and greater counts is greater than the database size, adjust
        // the counts based on their ratio to the sum to ensure that the sum is less than the
        // database size.
        let sum_after_noise = less + greater;
        if sum_after_noise > self.client.database.len() {
            let diff = sum_after_noise - self.client.database.len();
            // Compute the adjustment w.r.t to the ratio of the count to the current sum.
            let less_adjustment = (less as f64 / sum_after_noise as f64 * diff as f64).round() as usize;
            let greater_adjustment = (greater as f64 / sum_after_noise as f64 * diff as f64).round() as usize;
            less = less.saturating_sub(less_adjustment);
            greater = greater.saturating_sub(greater_adjustment);
        }

        let now = Instant::now();
        let [lt, gt] = self.client.local_comp2(less, greater);
        tracing::trace!(elapsed_ms = %now.elapsed().as_millis(), "local computation encryption");
        [lt, gt]
    }

    fn compute_shares(&self, lt: Ciphertext, gt: Ciphertext) -> [PartialDecryption; 2] {
        self.client.compute_shares(lt, gt)
    }

    fn update_search_range(&mut self, update: UpdateSearchRange) -> Option<T> {
        self.client.update_search_range(update)
    }
}

impl<T> DPClient<T> where T: TypeTrait {
    pub fn new(client: PartyClient<T>, get_scale_fn: GetScaleFn, noise_level: NoiseLevel) -> Self {
        DPClient {
            client,
            noise_level,
            get_scale_fn,
            noise_array: vec![],
        }
    }

    fn add_noise(&mut self, result: usize) -> usize {
        let mut rng = rand::thread_rng();

        // If we are searching for min/max, we make the comparisons within the search range instead
        // of the entire database. Therefore, we set the database size to the search range in this case.
        let db_size: usize = if self.client.k == 1 || self.client.k == self.client.databases_size {
            (self.client.search_range_idx.end - self.client.search_range_idx.start).max(1)
        } else {
            self.client.database.len() };

        let scale = (self.get_scale_fn)(self.noise_level, result, db_size);

        let noise = laplace_point(&mut rng, scale);
        self.noise_array.push(noise);
        let noise = noise.round() as isize;
        ((result as isize) + noise).max(0) as usize
    }
}

fn laplace_point<R: Rng>(rng: &mut R, scale: f64) -> f64 {
    let dist = Uniform::new(0.0, 1.0);
    let p: f64 = dist.sample(rng);
    // Apply the inverse CDF to the uniformly distributed variable.
    // F^-1(p) = \mu -\sigma \sgn(p - 0.5) \ln(1 - 2 |p - 0.5|)
    let mu = 0.0;
    let laplace_point = mu - scale * (p - 0.5).signum() * (1.0 - 2.0 * (p - 0.5).abs()).ln();
    laplace_point
}

// Computes the range for the laplacian noise given the result. Is currently not used.
fn get_noise_range(result: usize) -> (f64, f64) {
    // Compute the range as being at most 10% of the result.
    let range = ((result as f64) * 0.1).max(1.0);
    (-range, range)
}


#[test]
fn test_laplace_points() {
    let mut rng = rand::thread_rng();
    let db_size = 10;

    for _ in 0..10000 {
        let result = rng.gen_range(0..10);
        let scale = get_scale_sigmoid(NoiseLevel::MEDIUM, result, db_size);
        // Round it to a whole number.
        let laplace_point = laplace_point(&mut rng, scale);
        print!("{} ", laplace_point);
    }
}
