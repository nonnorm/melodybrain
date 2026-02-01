use std::{
    collections::HashMap,
    fs::OpenOptions,
    net::{IpAddr, Ipv6Addr, UdpSocket},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use maxminddb::PathElement;
use melodybrain::{Heartbeat, Stats, StoredIpStats, WORLDWIDE, search_country};

use crate::dbs::{GeneralIpDb, GeoIpDb};

mod dbs;

#[derive(Debug)]
struct AddrInfo {
    last_seen: Instant,
    country: Option<u16>,
}

fn main() {
    let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 2026)).unwrap();
    let geoip = GeoIpDb::new();
    let mut db = GeneralIpDb::new();

    let mut buf = [0; 32];

    let mut start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Initial cleanup run in case server was shut down
    db.cleanup(start);

    loop {
        let Ok((n, addr)) = socket.recv_from(&mut buf) else {
            continue;
        };

        // Only IPv4 for now, consider using a more advanced structure in the future as a DB
        let IpAddr::V4(addr_v4) = addr.ip().to_canonical() else {
            continue;
        };

        let Ok(heartbeat) = postcard::from_bytes::<Heartbeat>(&buf[..n]) else {
            continue;
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let bucket_info = db.lookup_ip(addr_v4);

        if bucket_info.last_seen == 0 {
            bucket_info.first_seen = now;
            bucket_info.country = geoip.lookup_ip(addr.ip()).unwrap_or_default();
        }

        if now - bucket_info.last_seen > 10 {
            bucket_info.last_seen = now;

            let country = bucket_info.country;
            if country != 0 {
                let country_seed = db.lookup_country(country);
                country_seed.seed += (heartbeat.seed as i64 - country_seed.seed) / 2000;
            }

            let global_seed = db.lookup_country(WORLDWIDE);
            global_seed.seed += (heartbeat.seed as i64 - global_seed.seed) / 2000;
        }

        if heartbeat.wants_country != 0 {
            let wants_seed = db.lookup_country(heartbeat.wants_country).seed;

            let stats = Stats {
                connected: 0,
                seed: wants_seed as i32,
            };
            let stats = postcard::to_slice(&stats, &mut buf).unwrap();
            let _ = socket.send_to(stats, addr);
        }

        if now - start > 20 {
            db.cleanup(now);
            start = now;
        }
    }
}
