use std::error::Error;
use std::marker::PhantomData;

use bincode::deserialize;
use pht_crypto::{Ciphertext, paillier::PartialDecryption};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::net::netserver::ServerMessage;
use crate::party::{TypeTrait, UpdateSearchRange};
use crate::party::party_client::PartyClientTrait;

#[derive(Serialize, Deserialize)]
pub(crate) enum ClientMessage {
    MsgCiphertext(Ciphertext, Ciphertext),
    MsgPartialDecryption(PartialDecryption, PartialDecryption),
}

pub(crate) fn parse_client_message(data: &[u8]) -> Result<ClientMessage, Box<dyn Error>> {
    // Deserialize the received data into one of the known message types
    deserialize(data).map_err(|e| e.into())
}

pub(crate) struct NetworkClient<T, C> where T: TypeTrait, C: PartyClientTrait<T> {
    client: C,
    // The cryptographic party client implementation
    stream: TcpStream,
    // The TCP connection to the server
    phantom: PhantomData<T>,
}

impl<T, C> NetworkClient<T, C>
    where
        T: TypeTrait,
        C: PartyClientTrait<T>,
{
    // Create a new NetworkClient instance and establish a TCP connection to the specified server address.
    pub async fn new(client: C, server_addr: &str) -> Result<Self, Box<dyn Error>> where T: TypeTrait, C: PartyClientTrait<T> {
        // Establish a TCP connection to the specified server address
        let stream = TcpStream::connect(server_addr).await?;
        // Return the constructed NetworkClient instance
        Ok(Self { client, stream, phantom: Default::default() })
    }

    pub async fn run_protocol(&mut self) -> Result<T, Box<dyn Error>> {
        loop {
            let [lt, gt] = self.client.local_computation();
            let msg = ClientMessage::MsgCiphertext(lt, gt);
            let msg_bytes = bincode::serialize(&msg).unwrap();
            self.send_data_to_server(msg_bytes.as_slice()).await?;


            let data = self.receive_data_from_server().await?;
            let msg = bincode::deserialize::<ServerMessage>(&data).unwrap();
            match msg {
                ServerMessage::MsgDecryptRequest(sum_lt_enc, sum_gt_enc) => {
                    let [sum_lt, sum_gt] = self.client.compute_shares(sum_lt_enc, sum_gt_enc);
                    let msg = ClientMessage::MsgPartialDecryption(sum_lt, sum_gt);
                    let msg_bytes = bincode::serialize(&msg).unwrap();
                    self.send_data_to_server(msg_bytes.as_slice()).await?;
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }

            let data = self.receive_data_from_server().await?;
            let msg = bincode::deserialize::<ServerMessage>(&data).unwrap();
            match msg {
                ServerMessage::MsgUpdateSearchRange(update) => {
                    match update {
                        UpdateSearchRange::Abort => {
                            return Ok(T::from(-1)); //FIXME
                        }
                        _ => {
                            let res = self.client.update_search_range(update);
                            if res.is_some() {
                                return Ok(res.unwrap());
                            }
                        }
                    }
                }
                _ => {
                    panic!("Unexpected message type");
                }
            }

            tracing::debug!("Next iteration");
        }
        // Return an error:
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Protocol failed")))
    }

    // Send the given data to the server over the TCP connection
    async fn send_data_to_server(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        // Write the data to the TCP stream, sending it to the server
        self.stream.write_all(data).await?;
        Ok(())
    }

    // Receive data from the server over the TCP connection
    async fn receive_data_from_server(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = vec![0u8; 1024];
        let mut reader = BufReader::new(&mut self.stream);
        let n = reader.read(&mut buf).await.unwrap();
        // If less than 1024 bytes were read, resize the buffer to the actual amount read
        buf.truncate(n);

        tracing::debug!("Received {} bytes from Server", buf.len());

        // Return the received data
        Ok(buf)
    }
}
