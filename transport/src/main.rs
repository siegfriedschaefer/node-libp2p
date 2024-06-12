use libp2p::{
    futures::StreamExt, swarm::dummy::Behaviour as DummyBehaviour, swarm::Swarm, Multiaddr, SwarmBuilder
};

use tokio::{io, io::AsyncBufReadExt, select};


fn create_swarm() -> Swarm<DummyBehaviour> {
    SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            Default::default(),
            libp2p::tls::Config::new,
            libp2p::yamux::Config::default,
        )
        .unwrap()
        .with_behaviour(|_| DummyBehaviour)
        .unwrap()
        .build()
}

#[tokio::main]
async fn main() {
    println!("chat example");
    let mut swarm0 = create_swarm();

    swarm0
    .listen_on("/ip4/127.0.0.1/tcp/8888".parse().unwrap())
    .unwrap();

    while let Some(event) = swarm0.next().await {
        println!("swarm0: {:?}", event);
    }


    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Kick it off
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                println!("Sending message: {:?}", line);
            }
        }
    }

    /*
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
*/


}
