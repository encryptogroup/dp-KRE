use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use futures::future::join_all;
use pht_crypto::{Ciphertext, paillier::PartialDecryption};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::net::netclient::ClientMessage;
use crate::net::netclient::parse_client_message;
use crate::party::party_server::PartyServerTrait;
use crate::party::UpdateSearchRange;

#[derive(Serialize, Deserialize)]
pub(crate) enum ServerMessage {
    MsgDecryptRequest(Ciphertext, Ciphertext),
    MsgUpdateSearchRange(UpdateSearchRange),
}

pub(crate) struct NetworkServer<S> where S: PartyServerTrait {
    server: S,
    listener: TcpListener,
    clients: Vec<Arc<Mutex<TcpStream>>>,
    num_clients: usize,
}

impl<S> NetworkServer<S>
    where
        S: PartyServerTrait,
{
    // Constructor to create a new NetworkServer instance
    pub async fn new(address: &str, server: S, num_clients: usize) -> Result<Self, Box<dyn Error>>
        where S: PartyServerTrait {
        // Bind a TCP listener to the specified address to accept incoming connections
        let listener = TcpListener::bind(address).await?;
        let clients: Vec<Arc<Mutex<TcpStream>>> = Vec::new();
        // Return the constructed NetworkServer instance
        Ok(Self { server, listener, clients, num_clients })
    }

    // Main loop to accept incoming client connections
    pub async fn init_connections(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            // Accept a new client connection, getting a TcpStream for the client
            let (socket, _) = self.listener.accept().await?;

            // Lock the mutex around the client list, getting a mutable reference to the vector
            // let mut clients_guard = self.clients.lock().await;
            // Add the new client to the vector inside self.clients
            self.clients.push(Arc::from(Mutex::from(socket)));

            if self.clients.len() == self.num_clients {
                // If the number of clients has reached the expected number, then we can start the protocol
                break;
            }
        }
        tracing::trace!("All clients connected");
        Ok(())
    }

    pub(crate) async fn run_protocol(&mut self) {
        loop {
            // Initialize a vector of Ciphertext with the desired size.
            let mut lt_array_cipher: Vec<Ciphertext> = vec![Ciphertext::from(0); self.num_clients];
            let mut gt_array_cipher: Vec<Ciphertext> = vec![Ciphertext::from(0); self.num_clients];

            // Get the timestamp before spawning tasks
            let start_time = Instant::now();

            let (tx, mut rx) = mpsc::channel(self.num_clients);

            let handles: Vec<tokio::task::JoinHandle<_>> = self.clients.iter().cloned().enumerate().map(|(id, client)| {
                let client = client.clone();
                let tx = tx.clone();  // Clone the transmitter for each client

                tokio::spawn(async move {
                    let mut client = client.lock().await;
                    let mut reader = BufReader::new(&mut *client);

                    // Read the data into a buffer of 1024 bytes
                    let mut buf = vec![0u8; 128];
                    let n = reader.read(&mut buf).await.unwrap();
                    buf.truncate(n);

                    tracing::trace!("Received {} bytes from Client (MsgCiphertext)", buf.len());

                    let parsed_message = parse_client_message(&buf).unwrap();
                    match parsed_message {
                        ClientMessage::MsgCiphertext(ciphertext1, ciphertext2) => {
                            tx.send((id, ciphertext1, ciphertext2)).await.expect("Failed to send");
                        }
                        _ => {
                            panic!("Unexpected message type");
                        }
                    }
                })
            }).collect();

            for _ in 0..self.num_clients {
                if let Some((id, ciphertext1, ciphertext2)) = rx.recv().await {
                    lt_array_cipher[id] = ciphertext1;
                    gt_array_cipher[id] = ciphertext2;
                }
            }

            join_all(handles).await;

            // Get the timestamp after all tasks have completed
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time);

            tracing::debug!("Total duration for reading Ciphertexts: {:?}", duration);

            // You can now safely read from lt_array and gt_array
            let [sum_lt_enc, sum_gt_enc] = self.server.add_ciphertexts(&lt_array_cipher, &gt_array_cipher);

            // Broadcast the sum to all clients
            let msg = ServerMessage::MsgDecryptRequest(sum_lt_enc, sum_gt_enc);
            let msg_bytes = bincode::serialize(&msg).unwrap();
            self.broadcast_to_all_parties(msg_bytes.as_slice()).await.unwrap();

            // NEXT STATE!!!!

            // Initialize a vector of Option<PartialDecryption> with the desired size.
            let mut lt_array_decrypt: Vec<PartialDecryption> = Vec::new();
            let mut gt_array_decrypt: Vec<PartialDecryption> = Vec::new();

            // Get the timestamp before spawning tasks
            let start_time = Instant::now();

            let (tx, mut rx) = mpsc::channel(self.num_clients);

            let handles: Vec<tokio::task::JoinHandle<_>> = self.clients.iter().cloned().enumerate().map(|(id, client)| {
                let tx = tx.clone();  // Clone the transmitter for each client

                tokio::spawn(async move {
                    let mut client = client.lock().await;
                    let mut reader = BufReader::new(&mut *client);

                    // Read the data into a buffer of 1024 bytes
                    let mut buf = vec![0u8; 128];
                    let n = reader.read(&mut buf).await.unwrap();
                    buf.truncate(n);

                    tracing::debug!("Received {} bytes from Client (MsgPartialDecryption)", buf.len());

                    let parsed_message = parse_client_message(&buf).unwrap();
                    match parsed_message {
                        ClientMessage::MsgPartialDecryption(decryption1, decryption2) => {
                            tx.send((id, decryption1, decryption2)).await.expect("Failed to send");
                        },
                        _ => {
                            panic!("Unexpected message type");
                        }
                    }
                })
            }).collect();

            for _ in 0..self.num_clients {
                if let Some((id, decryption1, decryption2)) = rx.recv().await {
                    lt_array_decrypt.push(decryption1);
                    gt_array_decrypt.push(decryption2);
                }
            }

            join_all(handles).await;

            // Get the timestamp after all tasks have completed
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time);

            tracing::debug!("Total duration for reading Decrypted data: {:?}", duration);


            let sums = self.server.combine_shares(&lt_array_decrypt, &gt_array_decrypt);
            let update = self.server.calculate_update(sums);

            let msg = ServerMessage::MsgUpdateSearchRange(update);
            let msg_bytes = bincode::serialize(&msg).unwrap();
            self.broadcast_to_all_parties(msg_bytes.as_slice()).await.unwrap();

            match update {
                UpdateSearchRange::FoundK => {
                    tracing::debug!("Protocol finished. Found k.");
                    break
                }
                UpdateSearchRange::Abort => {
                    tracing::debug!("Communication error. Aborted.");
                    break
                }
                _ => {
                    tracing::debug!("Protocol continues. Did not find k.");
                }
            }
        }
    }

    //Broadcast the given data to all connected clients
    async fn broadcast_to_all_parties(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send>> {
        // Use futures::future::join_all to run all send operations in parallel
        let send_futures: Vec<_> = self.clients.iter().cloned().map(|client| {
            let data = data.to_vec();  // Clone the data for each client
            tokio::spawn(async move {
                let mut locked_client = client.lock().await;
                let mut writer = BufWriter::new(&mut *locked_client);
                writer.write_all(&data).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                writer.flush().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                Ok::<(), Box<dyn std::error::Error + Send>>(())
            })
        }).collect();

        let results: Vec<_> = join_all(send_futures).await;

        // Check if all sends were successful
        for result in results {
            match result {
                Ok(Ok(_)) => {}, // Successful send
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(Box::new(e)), // Join error (panic in task)
            }
        }

        Ok(())
    }
}
