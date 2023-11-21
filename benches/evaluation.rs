use std::path::Path;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput, PlottingBackend};
use tracing_subscriber::fmt::format;
use privdev_dp_comp::party::dp_client::{get_scale_fixed, get_scale_sigmoid, GetScaleFn, NoiseLevel};
use privdev_dp_comp::utils::protocol::{create_server_clients, create_server_dp_clients, KValue, sample_databases};
use privdev_dp_comp::protocol::leaky_kth_ranked_element;


const DB_SIZES: [usize; 5] = [10000, 50000, 100000, 300000, 500000];
const DB_SIZE_DP: usize = 100000;
const MIN_DB_VALUE: i32 = -1000;
const MAX_DB_VALUE: i32 = 1000;
const NUM_PARTIES: usize = 10;

// The scale function used for the Laplacian noise.
const NOISE_SCALE_FN: GetScaleFn = get_scale_fixed;

/// The benchmark function for testing the original protocol with leakage with different db sizes.
fn protocol_leakage(c: &mut Criterion) {
    let mut group = c.benchmark_group("Protocol Leakage");
    for db_size in DB_SIZES {
        let databases = sample_databases::<i32>(db_size, NUM_PARTIES, MIN_DB_VALUE, MAX_DB_VALUE);
        for k in [KValue::Min, KValue::Median, KValue::Max] {
                group.bench_with_input(BenchmarkId::new(format!("db_size={}", db_size), format!("k={}", k)), &db_size, |b, &db_size| {
                b.iter(|| {
                    let (mut server, mut parties) = create_server_clients(k.to_k(db_size), databases.clone());
                    leaky_kth_ranked_element(&mut server, &mut parties)
                });
            });
        }

    }
    group.finish();
}

/// The benchmark function for testing the protocol with DP with different noise levels and for different k values.
fn protocol_dp(c: &mut Criterion) {
    let mut group = c.benchmark_group("Protocol DP");
    let databases = sample_databases::<i32>(DB_SIZE_DP, NUM_PARTIES, MIN_DB_VALUE, MAX_DB_VALUE);

    for level in [NoiseLevel::NONE, NoiseLevel::LOW, NoiseLevel::MEDIUM, NoiseLevel::HIGH] {
        for k in [KValue::Min, KValue::Median, KValue::Max] {
            group.bench_with_input(BenchmarkId::new(format!("k={}", k), format!("{} noise", level)), &level, |b, &level| {
                b.iter(|| {
                    let (mut server, mut parties) = create_server_dp_clients(k.to_k(DB_SIZE_DP), databases.clone(), NOISE_SCALE_FN, level);
                    leaky_kth_ranked_element(&mut server, &mut parties)
                });
            });
        }
    }
    group.finish();
}

criterion_group! {name=protocol_leak_bench;
    config=get_criterion();
    targets=protocol_leakage
}

criterion_group! {name=protocol_dp_bench;
    config=get_criterion();
    targets=protocol_dp
}

criterion_main!(protocol_leak_bench, protocol_dp_bench);


fn get_criterion() -> Criterion {
    Criterion::default().plotting_backend(PlottingBackend::Plotters).output_directory(Path::new("./bench_results"))
}