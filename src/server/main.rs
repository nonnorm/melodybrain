use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket},
    time::{Duration, Instant},
};

use melodybrain::{Heartbeat, Stats};

fn main() {
    let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 2026)).unwrap();

    let mut buf = [0; 32];

    let mut active_addrs = HashMap::with_capacity(8192);
    // Seed is stored as an i64 to prevent any overflow issues
    let mut seed = 0_i64;

    let mut start = Instant::now();

    loop {
        let Ok((n, addr)) = socket.recv_from(&mut buf) else {
            continue;
        };

        let Ok(Heartbeat(new_seed)) = postcard::from_bytes(&buf[..n]) else {
            continue;
        };

        let now = Instant::now();

        if active_addrs
            .get(&addr.ip())
            .is_some_and(|&x| now.duration_since(x) < Duration::from_secs(10))
        {
            continue;
        }

        if seed == 0 {
            seed = new_seed as i64;
        } else {
            seed = seed + (new_seed as i64 - seed) / 2000
        }

        let stats = Stats {
            connected: active_addrs.len() as u32,
            seed: seed as i32,
        };
        let stats = postcard::to_slice(&stats, &mut buf).unwrap();
        let _ = socket.send_to(stats, addr);

        active_addrs.insert(addr.ip(), Instant::now() + Duration::from_secs(30));

        if now.duration_since(start) > Duration::from_secs(20) {
            start = now;
            active_addrs.retain(|_, v| *v > start);
        }
    }
}
