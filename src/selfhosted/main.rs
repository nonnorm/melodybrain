use std::sync::{Arc, atomic::AtomicI32};
use tokio::net::{TcpListener, UdpSocket};

mod http;
mod notes;
mod udp;

#[derive(Debug)]
pub struct State {
    pub sock: UdpSocket,
    pub local_seed: AtomicI32,
}

fn generate_seed() -> i32 {
    let mut bytes = [0; 4];
    getrandom::fill(&mut bytes).expect("os rng error");
    i32::from_ne_bytes(bytes)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:33445")
        .await
        .expect("failed to bind listener to port 33445");

    let connector = UdpSocket::bind("0.0.0.0:0")
        .await
        .expect("failed to bind UDP socket");

    connector
        .connect("ravenclaw900.duckdns.org:2026")
        // .connect("localhost:2026")
        .await
        .expect("failed to connect to main server - run your own perhaps ;)");

    let state = Arc::new(State {
        sock: connector,
        local_seed: AtomicI32::new(generate_seed()),
    });

    tokio::spawn(udp::heartbeats(Arc::clone(&state)));

    axum::serve(listener, http::router(state))
        .await
        .expect("failed to start http listener");
}
