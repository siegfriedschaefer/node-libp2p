
use std::{
    path::PathBuf,
    time::Duration,
    collections::HashSet
};

use env_logger::Env;
use clap::Parser;
use futures::StreamExt;

use tokio::signal::unix::{signal, SignalKind};

use mypai_network_transport::{
    util::{addr_is_reachable, get_keypair},
    protocol::{dht_protocol, ID_PROTOCOL},
    QuicConfig,
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
    swarm::SwarmEvent,
    SwarmBuilder,
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

    // Start the swarm
    let quic_config = QuicConfig::from_env();
    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_quic_config(|config| config.mtu_upper_bound(quic_config.mtu_discovery_max))
        .with_dns()?
        .with_behaviour(behaviour)
        .expect("infallible")
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(120)))
        .build();
    for listen_addr in listen_addrs {
        log::info!("Listening on {}", listen_addr);
        swarm.listen_on(listen_addr)?;
    }
    for public_addr in cli.transport.p2p_public_addrs {
        log::info!("Adding public address {public_addr}");
        swarm.add_external_address(public_addr);
    }

    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

/*
    we do not support any contracts right now.
    let mut registered_nodes = HashSet::new();
    let mut nodes_stream = contract_client.network_nodes_stream(Duration::from_secs(300)).fuse();
*/

    loop {
        let event = tokio::select! {
            event = swarm.select_next_some() => event,
    /*
            res = nodes_stream.select_next_some() => {
                let nodes = match res {
                    Err(e) => {
                        log::error!("Error retrieving registered nodes from chain: {e:?}");
                        continue;
                    }
                    Ok(nodes) if nodes == registered_nodes => {
                        log::debug!("Registered nodes set unchanged.");
                        continue;
                    }
                    Ok(nodes) => nodes,
                };
                log::info!("Updating registered nodes");
                // Disallow nodes which are no longer registered
                for peer_id in registered_nodes.difference(&nodes) {
                    log::info!("Blocking peer {peer_id}");
                    swarm.behaviour_mut().allow.disallow_peer(*peer_id);
                }
                // Allow newly registered nodes
                for peer_id in nodes.difference(&registered_nodes) {
                    log::info!("Allowing peer {peer_id}");
                    swarm.behaviour_mut().allow.allow_peer(*peer_id);
                }
                registered_nodes = nodes;
                continue;
            }
    */
            _ = sigint.recv() => break,
            _ = sigterm.recv() => break,
        };


        log::trace!("Swarm event: {event:?}");
        if let SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
            peer_id,
            info: identify::Info { listen_addrs, .. },
        })) = event
        {
            listen_addrs.into_iter().filter(addr_is_reachable).for_each(|addr| {
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
            });
        }
    }

    log::info!("Bootnode stopped");

    Ok(())
}