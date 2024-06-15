
use libp2p_swarm_derive::NetworkBehaviour;

#[derive(NetworkBehaviour)]
pub struct InnerBehaviour {
    identify: identify::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
    relay: relay::client::Behaviour,
    dcutr: dcutr::Behaviour,
    ping: ping::Behaviour,
    autonat: autonat::Behaviour,
    allow: allow_block_list::Behaviour<AllowedPeers>,
    pubsub: Wrapped<PubsubBehaviour>,
}
