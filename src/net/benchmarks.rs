use std::mem;
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use tokio::sync::Mutex;
use tokio::test;

use crate::net::netclient::NetworkClient;
use crate::net::netserver::NetworkServer;
use crate::party::dp_client::{DPClient, get_scale_sigmoid, NoiseLevel};
use crate::party::party_client::PartyClientTrait;
use crate::party::party_server::{PartyServer, PartyServerTrait};
use crate::party::TypeTrait;
use crate::test::init_logging;
use crate::utils::protocol::{create_server_dp_clients, get_kth_element, KValue, sample_databases};

const SERVER_ADDRESS: &str = "localhost:8080";


const NUM_PARTIES: usize = 100;
const DB_SIZE: usize = 100;
const k: KValue = KValue::Median;

pub async fn kre_protocol_net<T, P, S>(server: PartyServer, parties: Vec<P>) -> T
    where
        T: TypeTrait + 'static,
        P: PartyClientTrait<T> + 'static,
        S: PartyServerTrait,
{
    // Create a new network server instance
    let mut server = NetworkServer::<PartyServer>::new(
        SERVER_ADDRESS, server, parties.len()).await.unwrap();


    let result = Arc::new(Mutex::new(None));

    let mut handles = Vec::new();
    for party in parties.into_iter() {
        let result = result.clone();
        let handle = tokio::spawn(async move {
            let mut client = NetworkClient::<T, P>::new(party, SERVER_ADDRESS).await.unwrap();
            let output = client.run_protocol().await.unwrap();
            let mut result = result.lock().await;
            if result.is_none() {
                *result = Some(output);
            }
        });

        handles.push(handle);
    }

    server.init_connections().await.unwrap();

    server.run_protocol().await;
    join_all(handles).await;

    tracing::debug!("Found kth-ranked element: {:?}", result);
    let mut result_guard = result.lock().await;
    let inner_value = mem::replace(&mut *result_guard, None);
    inner_value.unwrap()
}

// simple test
#[test]
async fn average_test_multi_party() {
    init_logging();
    let mut durations: Vec<Duration> = Vec::new();

    while durations.len() < 10 {
        test_multi_party(&mut durations).await;
    }

    let total_duration: Duration = durations.iter().sum();
    let average_duration = total_duration / 10; // As we have 10 correct runs.

    tracing::info!("Average time for 10 correct runs: {:?}", average_duration);
}

async fn test_multi_party(durations: &mut Vec<Duration>) {
    let databases = sample_databases::<i32>(DB_SIZE,NUM_PARTIES, 0, 1000);
    let res_exp = get_kth_element(&databases, k.to_k(DB_SIZE));

    let (server, parties) = create_server_dp_clients(k.to_k(DB_SIZE), databases, get_scale_sigmoid, NoiseLevel::LOW);
    let start = std::time::Instant::now();
    let res = kre_protocol_net::<i32, DPClient<i32>, PartyServer>(server, parties).await;
    let elapsed = start.elapsed();

    assert_eq!(res, res_exp);
    tracing::info!("Run: {:?}", elapsed);
    durations.push(elapsed);
}
