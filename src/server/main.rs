use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr, UdpSocket},
    time::{Duration, Instant},
};

use maxminddb::PathElement;
use melodybrain::{COUNTRY_CODES, Heartbeat, Stats, encode_code};

#[derive(Debug)]
struct AddrInfo {
    last_seen: Instant,
    country: Option<u16>,
}

fn main() {
    let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 2026)).unwrap();
    let geoip = unsafe {
        maxminddb::Reader::open_mmap("./GeoLite2-Country.mmdb")
            .expect("failed to open IP geo database")
    };

    let mut buf = [0; 32];

    let mut active_addrs: HashMap<IpAddr, AddrInfo> = HashMap::with_capacity(8192);
    // Seeds are stored as an i64 to prevent any overflow issues
    let mut country_seeds: HashMap<u16, i64> =
        HashMap::from_iter(COUNTRY_CODES.iter().map(|&k| (k, 0)));
    let mut global_seed = 0_i64;

    let mut start = Instant::now();

    loop {
        let Ok((n, addr)) = socket.recv_from(&mut buf) else {
            continue;
        };

        let Ok(heartbeat) = postcard::from_bytes::<Heartbeat>(&buf[..n]) else {
            continue;
        };

        let now = Instant::now();

        let country = if let Some(addr_info) = active_addrs.get_mut(&addr.ip()) {
            if now.duration_since(addr_info.last_seen) > Duration::from_secs(10) {
                addr_info.last_seen = now;
            }

            addr_info.country
        } else {
            let country = geoip
                .lookup(addr.ip())
                .ok()
                .and_then(|res| {
                    res.decode_path::<&str>(&[
                        PathElement::Key("country"),
                        PathElement::Key("iso_code"),
                    ])
                    .unwrap()
                })
                .map(|code| encode_code(code.as_bytes()));

            active_addrs.insert(
                addr.ip(),
                AddrInfo {
                    last_seen: now,
                    country,
                },
            );

            country
        };

        global_seed += (heartbeat.seed as i64 - global_seed) / 2000;
        if let Some(country) = country
            && let Some(country_seed) = country_seeds.get_mut(&country)
        {
            *country_seed += (heartbeat.seed as i64 - *country_seed) / 2000;
        };

        let stats = if let Some(country_seed) = country_seeds.get(&heartbeat.wants_country) {
            dbg!(heartbeat.wants_country, country_seed);
            Stats {
                connected: active_addrs
                    .values()
                    .filter(|x| x.country == Some(heartbeat.wants_country))
                    .count() as u32,
                seed: *country_seed as i32,
            }
        } else {
            Stats {
                connected: active_addrs.len() as u32,
                seed: global_seed as i32,
            }
        };

        let stats = postcard::to_slice(&stats, &mut buf).unwrap();
        let _ = socket.send_to(stats, addr);

        if now.duration_since(start) > Duration::from_secs(20) {
            start = now;
            active_addrs.retain(|_, v| v.last_seen.duration_since(now) > Duration::from_secs(30));
        }
    }
}
