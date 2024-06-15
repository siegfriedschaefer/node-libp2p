use libp2p::StreamProtocol;
use crate::Network;

pub const ID_PROTOCOL: &str = "/mypai/1.0.0";


pub const fn dht_protocol(network: Network) -> StreamProtocol {
    match network {
        Network::Tethys => StreamProtocol::new("/subsquid/dht/tethys/1.0.0"),
        Network::Mainnet => StreamProtocol::new("/subsquid/dht/mainnet/1.0.0"),
        Network::Mypainet => StreamProtocol::new("/subsquid/dht/mypainet/1.0.0"),
    }
}