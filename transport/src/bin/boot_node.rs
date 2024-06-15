
use std::{
    path::PathBuf,
    time::Duration,
};

use env_logger::Env;
use clap::Parser;

use mypai_network_transport::{
    util::{get_keypair},
    protocol::{dht_protocol, ID_PROTOCOL},
    TransportArgs,
};

use libp2p::{
    allow_block_list,
    allow_block_list::AllowedPeers,
    autonat,
    identify,
    identity::Keypair,
    PeerId,
    kad::{self, store::MemoryStore, Mode},
    gossipsub::{self, MessageAuthenticity},
    ping,
    relay,
};

use libp2p_swarm_derive::NetworkBehaviour;
use libp2p_connection_limits::ConnectionLimits;


#[derive(Parser)]
#[command(version)]

struct Cli {
    #[command(flatten)]
    transport: TransportArgs,

    #[arg(long, env, value_delimiter = ',', help = "Allowed nodes")]
    allowed_nodes: Vec<PeerId>,
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
    relay: relay::Behaviour,
    gossipsub: gossipsub::Behaviour,
    ping: ping::Behaviour,
    autonat: autonat::Behaviour,
    conn_limits: libp2p_connection_limits::Behaviour,
    allow: allow_block_list::Behaviour<AllowedPeers>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logging and parse arguments
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    log::info!("Starting bootnode");
    
    let cli = Cli::parse();
    let listen_addrs = cli.transport.listen_addrs();
    let keypair = get_keypair(cli.transport.key).await?;

    let local_peer_id = PeerId::from(keypair.public());
    log::info!("Local peer ID: {local_peer_id}");

    // Prepare behaviour & transport
    let autonat_config = autonat::Config {
        timeout: Duration::from_secs(60),
        throttle_clients_global_max: 64,
        throttle_clients_peer_max: 16,
        ..Default::default()
    };


    let mut kad_config = kad::Config::new(dht_protocol(cli.transport.rpc.network));
    kad_config.set_replication_factor(20.try_into().unwrap());

    let behaviour = |keypair: &Keypair| Behaviour {
        identify: identify::Behaviour::new(
            identify::Config::new(ID_PROTOCOL.to_string(), keypair.public())
                .with_interval(Duration::from_secs(60))
                .with_push_listen_addr_updates(true),
        ),

        kademlia: kad::Behaviour::with_config(
            local_peer_id,
            MemoryStore::new(local_peer_id),
            kad_config,
        ),

        relay: relay::Behaviour::new(local_peer_id, Default::default()),
        gossipsub: gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            Default::default(),
        )
        .unwrap(),
        ping: ping::Behaviour::new(Default::default()),
        autonat: autonat::Behaviour::new(local_peer_id, autonat_config),

        conn_limits: libp2p_connection_limits::Behaviour::new(
            ConnectionLimits::default().with_max_established_per_peer(Some(3)),
        ),
        allow: Default::default(),
    };
 
    log::info!("Bootnode stopped");

    Ok(())
}