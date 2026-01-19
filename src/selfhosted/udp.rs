use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use melodybrain::{Heartbeat, Stats};
use tokio::{
    net::UdpSocket,
    time::{Interval, MissedTickBehavior, interval},
};

use crate::http::ArcState;

pub async fn heartbeats(state: ArcState) {
    let mut interval = interval(Duration::from_secs(15));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut buf = [0; 32];

    loop {
        interval.tick().await;

        let local_seed = state.local_seed.load(Ordering::Relaxed);
        let heartbeat = Heartbeat(local_seed);
        let msg = postcard::to_slice(&heartbeat, &mut buf).unwrap();
        let _ = state.sock.send(msg).await;

        let Ok(Ok(n)) =
            tokio::time::timeout(Duration::from_secs(5), state.sock.recv(&mut buf)).await
        else {
            continue;
        };

        let Ok(stats) = postcard::from_bytes::<Stats>(&buf[..n]) else {
            continue;
        };

        state.other_seed.store(stats.seed, Ordering::Relaxed);
        state.connected.store(stats.connected, Ordering::Relaxed);
    }
}
