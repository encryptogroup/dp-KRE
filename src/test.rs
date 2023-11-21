use std::sync::Once;

use tracing::Level;

static INIT: Once = Once::new();
const LOG_LEVEL: Level = Level::INFO;

/// Initializes the logging for the tests. Is idempotent.
pub (crate) fn init_logging() {
    INIT.call_once(|| {
        let subscriber = tracing_subscriber::fmt().with_max_level(LOG_LEVEL).finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    });
}

#[cfg(test)]
pub mod tests {
    use std::time::Instant;

    use num::integer::Average;
    use rug::Integer;
    use tracing::log::debug;

    use crate::party::party_client::PartyClientTrait;
    use crate::party::party_server::PartyServerTrait;
    use crate::protocol::leaky_kth_ranked_element;
    use crate::test::init_logging;
    use crate::utils::protocol::{create_server_clients, create_single_party, get_kth_element, KValue, sample_database, sample_databases};

    // The sizes of the databases for the single party test.
    const DATABASE_SIZE_SINGLE: usize = 100;
    // The total size of all databases for the multi party test.
    const DATABASES_SIZE_MULTI: usize = 100000;
    const MIN_DB_VALUE: i32 = -100;
    const MAX_DB_VALUE: i32 = 100;
    const NUM_PARTIES: usize = 10;



    #[test]
    fn test_single_party() {
        init_logging();
        let db = sample_database::<i32>(DATABASE_SIZE_SINGLE, MIN_DB_VALUE, MAX_DB_VALUE);
        debug!("Test Single Party with database size: {}", DATABASE_SIZE_SINGLE);

        let min = db.iter().min().unwrap();
        let max = db.iter().max().unwrap();
        let m = min.average_floor(max);
        let exp_sum_lt = db.iter().filter(|&el| *el < m).count();
        let exp_sum_gt = db.iter().filter(|&el| *el > m).count();

        let (mut s, mut p) = create_single_party(db);
        let [lt, gt] = p.local_computation();
        let [sum_lt_enc, sum_gt_enc] = s.add_ciphertexts(&[lt], &[gt]);
        let [lt_share, gt_share] = p.compute_shares(sum_lt_enc, sum_gt_enc);
        let [sum_lt, sum_gt] = s.combine_shares(&[lt_share], &[gt_share]);
        assert_eq!(sum_lt, Integer::from(exp_sum_lt));
        assert_eq!(sum_gt, Integer::from(exp_sum_gt));
    }

    #[test]
    fn test_multi_party() {
        init_logging();
        let now = Instant::now();

        for k in [KValue::Min, KValue::Median, KValue::Max] {
            let k = k.to_k(DATABASES_SIZE_MULTI);
            tracing::debug!("Test Multi Party with k={}", k);
            let databases = sample_databases::<i32>(DATABASES_SIZE_MULTI, NUM_PARTIES, MIN_DB_VALUE, MAX_DB_VALUE);
            let res_exp = get_kth_element(&databases, k);

            let (mut server, mut parties) = create_server_clients(k, databases);
            let res = leaky_kth_ranked_element(&mut server, &mut parties);

            assert_eq!(res, Some(res_exp));
        }

        let elapsed = now.elapsed();
        tracing::info!("Time elapsed for test_multi_party: {:?}", elapsed);

    }
}


