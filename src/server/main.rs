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

    let mut buf = [0; 1200];

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

        let bucket_info = db.lookup_ip_mut(addr_v4);

        if bucket_info.first_seen == 0 && bucket_info.last_seen == 0 {
            bucket_info.first_seen = now;
            bucket_info.last_seen = now;

            let country = geoip.lookup_ip(addr.ip()).unwrap_or_default();
            bucket_info.country = country;

            let country_stats = db.lookup_country_mut(country);
            country_stats.active += 1;
            country_stats.unique += 1;

            let global_stats = db.lookup_country_mut(WORLDWIDE);
            global_stats.active += 1;
            global_stats.unique += 1;
        } else if bucket_info.first_seen != 0 && bucket_info.last_seen == 0 {
            bucket_info.first_seen = now;
            bucket_info.last_seen = now;

            let country = bucket_info.country;
            let country_stats = db.lookup_country_mut(country);
            country_stats.active += 1;

            let global_stats = db.lookup_country_mut(WORLDWIDE);
            global_stats.active += 1;
        }

        // Reborrow to satisfy borrow checker
        let bucket_info = db.lookup_ip_mut(addr_v4);

        if now - bucket_info.last_seen > 10 {
            let diff = (now - bucket_info.last_seen) as u32;
            bucket_info.cum_duration += diff;

            bucket_info.last_seen = now;
            bucket_info.hits += 1;

            let country = bucket_info.country;
            let country_seed = db.lookup_country_mut(country);
            country_seed.seed += (heartbeat.seed as i64 - country_seed.seed) / 2000;
            country_seed.cum_duration += diff;

            let global_seed = db.lookup_country_mut(WORLDWIDE);
            global_seed.seed += (heartbeat.seed as i64 - global_seed.seed) / 2000;
            global_seed.cum_duration += diff;
        }

        if heartbeat.wants_country != 0 {
            let country = db.lookup_country_mut(heartbeat.wants_country);

            let stats = Stats {
                connected: country.active,
                seed: country.seed as i32,
                // It might be somewhat inefficient to recalculate this every time a request is made, but it's only 250 countries
                country_heatmap: db.get_country_heatmap(),
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
