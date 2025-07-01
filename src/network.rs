// Network layer for distributed consensus

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use tracing::{info, debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    VoteRequest(crate::raft::VoteRequest),
    VoteResponse(crate::raft::VoteResponse),
    AppendEntries(crate::raft::AppendEntriesRequest),
    AppendEntriesResponse(crate::raft::AppendEntriesResponse),
    Heartbeat { term: u64, leader_id: String },
}

pub struct NetworkLayer {
    bind_addr: SocketAddr,
    peers: Vec<String>,
}

impl NetworkLayer {
    pub fn new(bind_addr: String, peers: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let bind_addr = bind_addr.parse()?;
        Ok(Self { bind_addr, peers })
    }

    pub async fn start_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("Network server listening on {}", self.bind_addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            debug!("New connection from {}", addr);
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket).await {
                    warn!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }

    pub async fn send_message(
        &self,
        peer: &str,
        message: Message,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Sending message to peer: {}", peer);
        
        let mut stream = TcpStream::connect(peer).await?;
        let serialized = serde_json::to_string(&message)?;
        
        // Send message length first
        let len = serialized.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        
        // Send message
        stream.write_all(serialized.as_bytes()).await?;
        stream.flush().await?;
        
        debug!("Message sent successfully to {}", peer);
        Ok(())
    }

    pub async fn broadcast_message(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Broadcasting message to {} peers", self.peers.len());
        
        let mut tasks = Vec::new();
        
        for peer in &self.peers {
            let peer = peer.clone();
            let message = message.clone();
            
            let task = tokio::spawn(async move {
                // Create temporary network layer for sending
                let temp_net = NetworkLayer::new("0.0.0.0:0".to_string(), vec![]).unwrap();
                if let Err(e) = temp_net.send_message(&peer, message).await {
                    warn!("Failed to send message to peer {}: {}", peer, e);
                }
            });
            
            tasks.push(task);
        }
        
        // Wait for all sends to complete
        for task in tasks {
            let _ = task.await;
        }
        
        debug!("Broadcast completed");
        Ok(())
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut len_buf = [0u8; 4];
    socket.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    
    let mut buf = vec![0u8; len];
    socket.read_exact(&mut buf).await?;
    
    let message: Message = serde_json::from_slice(&buf)?;
    debug!("Received message: {:?}", message);
    
    // Process message based on type
    match message {
        Message::VoteRequest(_) => {
            debug!("Processing vote request");
            // In a real implementation, this would be handled by the consensus engine
        }
        Message::VoteResponse(_) => {
            debug!("Processing vote response");
        }
        Message::AppendEntries(_) => {
            debug!("Processing append entries");
        }
        Message::AppendEntriesResponse(_) => {
            debug!("Processing append entries response");
        }
        Message::Heartbeat { term, leader_id } => {
            debug!("Received heartbeat from leader {} at term {}", leader_id, term);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_network_layer_creation() {
        let network = NetworkLayer::new(
            "127.0.0.1:8080".to_string(),
            vec!["127.0.0.1:8081".to_string()],
        );
        assert!(network.is_ok());
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let message = Message::Heartbeat {
            term: 1,
            leader_id: "node-1".to_string(),
        };
        
        let serialized = serde_json::to_string(&message);
        assert!(serialized.is_ok());
        
        let deserialized: Result<Message, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}