use clap::Parser;
use tracing::{info, warn};
use std::time::Duration;
use tokio::time;

mod raft;
mod consensus;
mod network;

use consensus::{ConsensusEngine, ConsensusConfig};

#[derive(Parser)]
#[command(name = "distributed-consensus")]
#[command(about = "A modern distributed consensus system")]
struct Cli {
    /// Node ID
    #[arg(short, long, default_value = "node-1")]
    node_id: String,
    
    /// Listening port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    
    /// Peer addresses
    #[arg(short, long)]
    peers: Vec<String>,
    
    /// Algorithm to use
    #[arg(short, long, default_value = "raft")]
    algorithm: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();
    
    info!("Starting distributed consensus node: {}", cli.node_id);
    info!("Algorithm: {}", cli.algorithm);
    info!("Port: {}", cli.port);
    info!("Peers: {:?}", cli.peers);

    let config = ConsensusConfig {
        node_id: cli.node_id.clone(),
        bind_addr: format!("0.0.0.0:{}", cli.port),
        peers: cli.peers,
        algorithm: cli.algorithm,
        election_timeout: Duration::from_millis(150),
        heartbeat_interval: Duration::from_millis(50),
    };

    let mut engine = ConsensusEngine::new(config).await?;
    
    // Start the consensus engine
    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            warn!("Consensus engine error: {}", e);
        }
    });

    // Simulate some operations
    demo_operations().await;

    // Wait for the engine
    engine_handle.await?;

    Ok(())
}

async fn demo_operations() {
    info!("Running demo consensus operations...");
    
    // Simulate leader election
    time::sleep(Duration::from_secs(2)).await;
    info!("Simulating leader election...");
    
    // Simulate log replication
    time::sleep(Duration::from_secs(1)).await;
    info!("Simulating log replication...");
    
    // Simulate network partition recovery
    time::sleep(Duration::from_secs(1)).await;
    info!("Simulating network partition recovery...");
    
    info!("Demo operations completed");
}