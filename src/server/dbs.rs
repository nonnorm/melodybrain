use std::{
    fs::OpenOptions,
    net::{IpAddr, Ipv4Addr},
};

use bytemuck::{cast_slice_mut, from_bytes, from_bytes_mut};
use maxminddb::{PathElement, Reader};
use melodybrain::{COUNTRIES, StoredCountryStats, StoredIpStats, WORLDWIDE, search_country};
use memmap2::{Mmap, MmapMut};

pub struct GeoIpDb(Reader<Mmap>);

impl GeoIpDb {
    pub fn new() -> Self {
        let db = unsafe {
            maxminddb::Reader::open_mmap("./GeoLite2-Country.mmdb")
                .expect("failed to open IP geo database")
        };

        Self(db)
    }

    pub fn lookup_ip(&self, ip: IpAddr) -> Option<u8> {
        self.0
            .lookup(ip)
            .ok()
            .and_then(|res| {
                res.decode_path::<&str>(&[
                    PathElement::Key("country"),
                    PathElement::Key("iso_code"),
                ])
                .unwrap()
            })
            .and_then(search_country)
    }
}

pub struct GeneralIpDb(MmapMut);

impl GeneralIpDb {
    pub fn new() -> Self {
        let db = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open("./ipv4.bin")
            .expect("failed to create/open ip database");

        db.set_len((1 << 24) * 32).expect("failed to sparsify db");

        let db = unsafe {
            memmap2::MmapOptions::new()
                .no_reserve_swap()
                .map_mut(&db)
                .expect("failed to mmap sparse db")
        };

        Self(db)
    }

    fn split(&mut self) -> (&mut [StoredIpStats], &mut [StoredCountryStats]) {
        let start = const { Ipv4Addr::new(1, 0, 0, 0).to_bits() >> 8 } as usize;
        let end = const { Ipv4Addr::new(223, 255, 255, 255).to_bits() >> 8 } as usize;

        let (countries, ips) = self.0.split_at_mut(start);
        let ips: &mut [StoredIpStats] = cast_slice_mut(&mut ips[start..=end]);
        let countries: &mut [StoredCountryStats] = cast_slice_mut(&mut countries[0..start]);

        (ips, countries)
    }

    pub fn cleanup(&mut self, now: u64) {
        let (records, countries) = self.split();

        for record in records {
            if record.first_seen != 0 {
                let diff = now - record.last_seen;
                if diff > 10 {
                    record.cum_duration += diff as u32;
                    record.last_seen = 0;

                    let country = &mut countries[record.country as usize];
                    country.active = country.active.saturating_sub(1);
                    let global = &mut countries[WORLDWIDE as usize];
                    global.active = global.active.saturating_sub(1);
                }
            }
        }
    }

    pub fn lookup_ip_mut(&mut self, addr: Ipv4Addr) -> &mut StoredIpStats {
        let ip_bucket = addr.to_bits() >> 8;
        let start_idx = ip_bucket as usize * 32;
        let end_idx = start_idx + 32;

        from_bytes_mut(&mut self.0[start_idx..end_idx])
    }

    pub fn lookup_country_mut(&mut self, country: u8) -> &mut StoredCountryStats {
        let start_idx = country as usize * 32;
        let end_idx = start_idx + 32;

        from_bytes_mut(&mut self.0[start_idx..end_idx])
    }

    pub fn lookup_country(&self, country: u8) -> &StoredCountryStats {
        let start_idx = country as usize * 32;
        let end_idx = start_idx + 32;

        from_bytes(&self.0[start_idx..end_idx])
    }

    pub fn get_country_heatmap(&self) -> [f32; COUNTRIES.len()] {
        let world_total = self.lookup_country(WORLDWIDE).active as f32;

        std::array::from_fn(|idx| self.lookup_country(idx as u8).active as f32 / world_total)
    }

    // pub fn lookup_ip_and_country(
    //     &mut self,
    //     addr: Ipv4Addr,
    // ) -> (&mut StoredIpStats, &mut StoredCountryStats) {
    //     self.0.split_at_mut(mid)
    // }
}
