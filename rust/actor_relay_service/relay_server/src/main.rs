#![allow(non_snake_case, unused_variables, unused_mut, unused_imports)]

use derive_more::Display;
use futures::future::err;
use log::debug;
use paris::{error, info, warn, Logger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::process::id;
use std::sync::Arc;
use std::time::Duration;
use std::{clone, collections::HashSet};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, MutexGuard};
use tokio::time::timeout;
use uuid::Uuid;
type AnyResult = anyhow::Result<()>;

#[derive(Debug, Display, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ClientId(Uuid);

impl ClientId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn get(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientMessage {
    pub clientId: Option<ClientId>,
    pub clientOperation: ClientOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ServerOperation {
    ClientConnectApproved(ClientId),
    RequestCurrentTaskStep,
}

/* intended as a grouping of clients, so things like
    "every one of your own characters" or "all the characters in the raid".
    String currently for flexibility until I figure out something better.
    Intended such that each client "connects" to one or more rooms at a time
    and each room has 1 or more channels.  A message is sent to a Room/Channel combination
*/
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub struct Room(String);
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub struct Channel(String);
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq, Hash)]
pub struct Message(String);

impl Deref for Room {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Channel {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Message {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ClientOperation {
    ConnectAttempt,  // connects to a socket
    RoomJoin(Room),  // joins a room
    RoomLeave(Room), // leaves a room
    Disconnect,
    Message {
        room: Room,
        channel: Channel,
        message: Message,
    },
}

pub struct Client {
    pub tx: mpsc::UnboundedSender<String>,
    pub clientId: ClientId,
}

impl Client {
    fn new(clientId: ClientId, tx: mpsc::UnboundedSender<String>) -> Self {
        Self { tx, clientId }
    }
}

struct Server {
    pub clients: HashMap<ClientId, Client>,
    pub writers: HashMap<ClientId, WriteHalf<TcpStream>>,
    pub rooms: HashMap<Room, HashSet<ClientId>>,
}

pub const ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> AnyResult {
    let mut log = Logger::new();
    let listener = TcpListener::bind(&ADDR).await?;
    let server = Arc::new(Mutex::new(Server {
        clients: HashMap::new(),
        writers: HashMap::new(),
        rooms: HashMap::new(),
    }));

    //let mut clientStreams: HashMap<ClientId, Arc<Mutex<TcpStream>>> = HashMap::new();

    info!("Listening on {}", &ADDR);

    loop {
        let (mut stream, _) = listener.accept().await?;
        info!("Received connection attempt from client");

        let server = Arc::clone(&server);

        let (reader, writer) = tokio::io::split(stream);

        // Spawn task for each client connection
        tokio::spawn(async move {
            let (mut tx, mut rx) = mpsc::unbounded_channel();
            let clientId = ClientId::new();
            {
                let mut server = server.lock().await;
                server
                    .clients
                    .insert(clientId.clone(), Client::new(clientId.clone(), tx.clone()));
                server.writers.insert(clientId.clone(), writer);
            }

            // Spawn a task to listen for messages to send to the client
            let server_locked = Arc::clone(&server);
            let client_id_clone = clientId.clone();

            // Create a channel for sending dead client IDs
            let (dead_client_sender, mut dead_client_receiver) = tokio::sync::mpsc::channel(100);
            let server_locked_clone = Arc::clone(&server_locked);

            // Thread that writes incoming messages to client
            tokio::spawn(async move {
                // Wait for incoming message
                while let Some(message) = rx.recv().await {
                    // lock server mutex
                    let mut server = server_locked.lock().await;
                    // obtain writer access
                    if let Some(writer) = server.writers.get_mut(&client_id_clone) {
                        // Send actual TCP message
                        if let Err(e) = writer.write_all(message.as_bytes()).await {
                            error!(
                                "Failed to write message to client {}: {}",
                                clientId.clone().0,
                                e
                            );
                            // Send dead client ID to the removal task
                            if let Err(e) = dead_client_sender.send(client_id_clone).await {
                                error!("Failed to send dead client ID: {}", e);
                            }
                        }
                    }
                }
            });

            // Spawn a task for removing dead clients
            tokio::spawn(async move {
                while let Some(dead_client_id) = dead_client_receiver.recv().await {
                    let mut server = server_locked_clone.lock().await;
                    server.writers.remove(&dead_client_id);
                }
            });

            let mut buf = vec![];
            let mut reader = tokio::io::BufReader::new(reader);

            loop {
                buf.clear();
                let _ = reader.read_until(b'\n', &mut buf).await;

                info!("Deserializing from string");
                let json_message = String::from_utf8_lossy(&buf)
                    .trim()
                    .trim_end_matches('\n')
                    .to_string();
                info!("JSON Message is: {}", &json_message);
                let message: Result<ClientMessage, _> = serde_json::from_str(&json_message);
                match message {
                    Ok(client_message) => match client_message.clientOperation {
                        ClientOperation::Message {
                            room,
                            channel,
                            message,
                        } => {
                            // We need to send this client message out to every single stream in all the tokio spawns

                            // Clone the message here
                            info!(
                                "Received client message: {} to room: {} and channel: {} from id: {}",
                                &message, &room, &channel, &client_message.clientId.unwrap().0
                            );
                            // We need to send this client message out to every single stream in all the tokio spawns
                            let mut server_guard = server.lock().await;
                            let responseMsg = format!("{} RESPONSE", message.clone());
                            let mut dead_clients = Vec::new();

                            if let Some(clients_in_room) = server_guard.rooms.get(&room) {
                                for client_id in clients_in_room {
                                    if let Some(client) = server_guard.clients.get(client_id) {
                                        if let Err(e) = client.tx.send(responseMsg.clone()) {
                                            error!(
                                                "Failed to send message to client {}: {}",
                                                client_id, e
                                            );
                                            // Queue dead client for removal
                                            dead_clients.push(*client_id);
                                        } else {
                                            info!("Sent message to client: {}", client_id);
                                        }
                                    }
                                }
                            }

                            // Remove dead clients
                            for client_id in dead_clients {
                                server_guard.clients.remove(&client_id);
                            }
                        }
                        ClientOperation::Disconnect => {
                            info!("The client has terminated the connection.");
                            break;
                        }
                        ClientOperation::ConnectAttempt => {
                            info!("In ClientConnectAttempt");

                            let tx_clone = tx.clone();
                            {
                                let mut server = server.lock().await;
                                server.clients.insert(
                                    clientId.clone(),
                                    Client::new(clientId.clone(), tx_clone),
                                );
                            }

                            // send client its new ID back
                            let server_operation =
                                ServerOperation::ClientConnectApproved(clientId.clone());
                            let operation_json = match serde_json::to_string(&server_operation) {
                                Ok(str) => str,
                                Err(err) => {
                                    error!(
                                        "Error occurred when serializing server operation: {}",
                                        err
                                    );
                                    continue;
                                }
                            };

                            info!("Writing to client stream");
                            // Write to the client's stream within the server lock scope
                            let mut server = server.lock().await;
                            if let Some(writer) = server.writers.get_mut(&clientId) {
                                if let Err(e) = writer.write_all(operation_json.as_bytes()).await {
                                    error!("Failed to write to client {}: {}", clientId, e);
                                } else {
                                    info!("Sent client response");
                                }
                            } else {
                                error!("Writer for client ID not found");
                            }
                        }
                        ClientOperation::RoomJoin(room) => {
                            info!("Client {} joining room {}", clientId, room);
                            let mut server = server.lock().await;
                            server
                                .rooms
                                .entry(room.clone())
                                .or_default()
                                .insert(clientId);
                            info!("Client {} joined room {}", clientId, room);
                        }
                        ClientOperation::RoomLeave(room) => {
                            info!("Client {} leaving room {}", clientId, room);
                            let mut server = server.lock().await;
                            if let Some(clients_in_room) = server.rooms.get_mut(&room) {
                                clients_in_room.remove(&clientId);
                                info!("Client {} left room {}", clientId, room);
                            }
                        }
                    },
                    Err(err) => {
                        error!("Failed to parse the message: {}", err);
                        break;
                    }
                }
            }
        });
    }
}
