use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use melodybrain::{Heartbeat, Stats};
use tokio::{
    net::UdpSocket,
    time::{Interval, MissedTickBehavior, interval, timeout},
};

use crate::{State, http::ArcState};

impl State {
    pub async fn send_heartbeat(&self, country: u16) -> Stats {
        let mut buf = [0; 32];

        loop {
            let local_seed = self.local_seed.load(Ordering::Relaxed);
            let heartbeat = Heartbeat {
                seed: local_seed,
                wants_country: country,
            };
            let msg = postcard::to_slice(&heartbeat, &mut buf).unwrap();
            let _ = self.sock.send(msg).await;

            let Ok(Ok(n)) =
                tokio::time::timeout(Duration::from_secs(2), self.sock.recv(&mut buf)).await
            else {
                continue;
            };

            let Ok(stats) = postcard::from_bytes::<Stats>(&buf[..n]) else {
                continue;
            };

            return stats;
        }
    }
}

pub async fn heartbeats(state: ArcState) {
    let mut interval = interval(Duration::from_secs(15));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        interval.tick().await;

        state.send_heartbeat(0).await;
    }
}
