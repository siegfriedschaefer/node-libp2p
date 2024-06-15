use clap::{Args, ValueEnum};
use std::{path::PathBuf, str::FromStr};
use libp2p::{Multiaddr, PeerId};

use crate::Address;

#[derive(Args)]
pub struct RpcArgs {
    #[arg(
        long,
        env,
        help = "Blockchain RPC URL",
        default_value = "http://127.0.0.1:8545/"
    )]
    pub rpc_url: String,
    #[arg(
        long,
        env,
        help = "Layer 1 blockchain RPC URL. If not provided, rpc_url is assumed to be L1"
    )]
    pub l1_rpc_url: Option<String>,

    #[command(flatten)]
    contract_addrs: ContractAddrs,

    #[arg(long, env, help = "Network to connect to (mainnet or testnet)")]
    pub network: Network,
}

#[derive(Args)]
pub struct ContractAddrs {
    #[arg(long, env)]
    pub gateway_registry_contract_addr: Option<Address>,
    #[arg(long, env)]
    pub worker_registration_contract_addr: Option<Address>,
    #[arg(long, env)]
    pub network_controller_contract_addr: Option<Address>,
    #[arg(long, env)]
    pub allocations_viewer_contract_addr: Option<Address>,
    #[arg(long, env)]
    pub multicall_contract_addr: Option<Address>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum Network {
    Tethys,
    Mainnet,
    Mypainet
}

impl Network {
    pub fn gateway_registry_default_addr(&self) -> Address {
        match self {
            Self::Tethys => "0xAB46F688AbA4FcD1920F21E9BD16B229316D8b0a".parse().unwrap(),
            Self::Mainnet => "0x8A90A1cE5fa8Cf71De9e6f76B7d3c0B72feB8c4b".parse().unwrap(),
            Self::Mypainet =>  "0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead".parse().unwrap(),
        }
    }

    pub fn worker_registration_default_addr(&self) -> Address {
        match self {
            Self::Tethys => "0xCD8e983F8c4202B0085825Cf21833927D1e2b6Dc".parse().unwrap(),
            Self::Mainnet => "0x36E2B147Db67E76aB67a4d07C293670EbeFcAE4E".parse().unwrap(),
            Self::Mypainet =>  "0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead".parse().unwrap(),
        }
    }

    pub fn network_controller_default_addr(&self) -> Address {
        match self {
            Self::Tethys => "0x68Fc7E375945d8C8dFb0050c337Ff09E962D976D".parse().unwrap(),
            Self::Mainnet => "0x4cf58097D790B193D22ed633bF8b15c9bc4F0da7".parse().unwrap(),
            Self::Mypainet =>  "0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead".parse().unwrap(),
        }
    }

    pub fn allocations_viewer_default_addr(&self) -> Address {
        match self {
            Self::Tethys => "0xC0Af6432947db51e0C179050dAF801F19d40D2B7".parse().unwrap(),
            Self::Mainnet => "0x88CE6D8D70df9Fe049315fd9D6c3d59108C15c4C".parse().unwrap(),
            Self::Mypainet =>  "0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead".parse().unwrap(),
        }
    }

    pub fn multicall_default_addr(&self) -> Address {
        match self {
            Self::Tethys | Self::Mainnet | Self::Mypainet => {
                "0xcA11bde05977b3631167028862bE2a173976CA11".parse().unwrap()
            }
        }
    }
}

impl RpcArgs {
    pub fn gateway_registry_addr(&self) -> Address {
        self.contract_addrs
            .gateway_registry_contract_addr
            .unwrap_or_else(|| self.network.gateway_registry_default_addr())
    }

    pub fn worker_registration_addr(&self) -> Address {
        self.contract_addrs
            .worker_registration_contract_addr
            .unwrap_or_else(|| self.network.worker_registration_default_addr())
    }

    pub fn network_controller_addr(&self) -> Address {
        self.contract_addrs
            .network_controller_contract_addr
            .unwrap_or_else(|| self.network.network_controller_default_addr())
    }

    pub fn allocations_viewer_addr(&self) -> Address {
        self.contract_addrs
            .allocations_viewer_contract_addr
            .unwrap_or_else(|| self.network.allocations_viewer_default_addr())
    }

    pub fn multicall_addr(&self) -> Address {
        self.contract_addrs
            .multicall_contract_addr
            .unwrap_or_else(|| self.network.multicall_default_addr())
    }
}

#[derive(Args)]
pub struct TransportArgs {
    #[arg(short, long, env = "KEY_PATH", help = "Path to libp2p key file")]
    pub key: Option<PathBuf>,

    #[arg(
        long,
        env,
        help = "Addresses on which the p2p node will listen",
        value_delimiter = ',',
        num_args = 1..,
        default_value = "/ip4/0.0.0.0/udp/0/quic-v1"
    )]
    p2p_listen_addrs: Vec<Multiaddr>,

    #[arg(
        long,
        env,
        help = "Public address(es) on which the p2p node can be reached",
        value_delimiter = ',',
        num_args = 1..,
    )]
    pub p2p_public_addrs: Vec<Multiaddr>,

    #[arg(
        long,
        env,
        help = "Connect to boot node '<peer_id> <address>'.",
        value_delimiter = ',',
        num_args = 1..,
    )]
    pub boot_nodes: Vec<BootNode>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

impl TransportArgs {
    pub fn listen_addrs(&self) -> Vec<Multiaddr> {
        self.p2p_listen_addrs.clone()
    }
}

#[derive(Debug, Clone)]
pub struct BootNode {
    pub peer_id: PeerId,
    pub address: Multiaddr,
}

impl FromStr for BootNode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let peer_id = parts
            .next()
            .ok_or("Boot node peer ID missing")?
            .parse()
            .map_err(|_| "Invalid peer ID")?;
        let address = parts
            .next()
            .ok_or("Boot node address missing")?
            .parse()
            .map_err(|_| "Invalid address")?;
        Ok(Self { peer_id, address })
    }
}



