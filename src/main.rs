use serde::Deserialize;
use std::fs;
use std::io::BufReader;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;
use tracing::{debug, error, info, warn};
use url::Url;

// Given a file path, rewrite it's mirror.

async fn mirror_latency(h: &str) -> Option<Duration> {
    info!("Profiling - {} ...", h);

    let mut addrs: Vec<_> = format!("{}:443", h)
        .to_socket_addrs()
        .unwrap()
        .map(|sa| sa.ip())
        .collect();

    while let Some(addr) = addrs.pop() {
        debug!(?addr);

        let mut pinger = match surge_ping::pinger(addr).await {
            Ok(p) => p,
            Err(e) => {
                warn!(?e, "Error creating pinger");
                continue;
            }
        };

        pinger.timeout(Duration::from_secs(2));

        let mut times = Vec::new();
        for seq_cnt in 0..5 {
            match pinger.ping(seq_cnt).await {
                Ok((reply, dur)) => {
                    debug!("time={:?}", dur);
                    // debug!("{} bytes from {}: icmp_seq={} ttl={:?} time={:?}",
                    //    reply.size, reply.source, reply.sequence, reply.ttl, dur);
                    times.push(dur);
                }
                Err(e) => {
                    if matches!(e, surge_ping::SurgeError::Timeout { seq: _ }) {
                        debug!(?e);
                    } else {
                        warn!(?e, "Error during ping");
                    }
                }
            }
        }

        if times.len() < 3 {
            // Not enough times recorded.
            continue;
        }

        // Okay, we have times, lets goooooo
        let sum: Duration = times.iter().sum();
        return Some(sum / times.len() as u32);
    }

    None
}

fn rewrite_mirror(p: &Path, m: &Url, known_m: &[Url]) {
    let mut repo = match ini::Ini::load_from_file(p) {
        Ok(r) => {
            let mut dump: Vec<u8> = Vec::new();
            let _ = r.write_to(&mut dump);
            let mut dump = unsafe { String::from_utf8_unchecked(dump) };
            debug!(%dump);
            r
        }
        Err(e) => {
            warn!(?p, ?e, "Failed to load repo");
            return;
        }
    };

    // Iterate over the sections
    for (name, sect) in repo.iter_mut() {
        let mut baseurl = match sect.get("baseurl").and_then(|burl| Url::parse(burl).ok()) {
            Some(u) => u,
            None => {
                warn!(
                    "No baseurl, or invalid baseurl in {:?} {} - skipping",
                    p,
                    name.unwrap_or("global")
                );
                continue;
            }
        };

        debug!(%baseurl);

        // Baseurl must be in the set of known mirrors that we are allowed to rewrite.
        let mut contains = false;
        for km in known_m {
            if baseurl.host_str() == km.host_str() {
                contains = true;
            }
        }

        if !contains {
            info!(
                "Not updating {} - not a known mirror base",
                baseurl.as_str()
            );
            continue;
        }

        let _ = baseurl.set_port(m.port());
        let _ = baseurl.set_host(m.host_str());
        let _ = baseurl.set_scheme(m.scheme());

        debug!(%baseurl);

        sect.insert("baseurl", baseurl);
    }

    if let Err(e) = repo.write_to_file(p) {
        warn!(?p, "Unable to write repo configuration");
    }
}

#[derive(Debug, Deserialize)]
struct MirrorDefinitions {
    mirrors: Vec<Url>,
}

#[derive(StructOpt)]
struct Config {
    #[structopt(env = "MIRROR_DEFS", long = "mirror_defs", short = "m")]
    /// Override the defined set of mirrors available for profiling and usage.
    mirror_definitions: Option<PathBuf>,
    #[structopt(short = "x")]
    /// Do it - profile mirrors and update repos. Without this, a dry-run is performed.
    doit: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Mirror Magic ✨ ");

    let config = Config::from_args();

    let mirrors = config
        .mirror_definitions
        .and_then(|p| {
            fs::File::open(&p)
                .map_err(|e| {
                    warn!(?e, ?p, "Unable to open");
                })
                .ok()
                .map(|f| BufReader::new(f))
                .and_then(|rdr| {
                    serde_json::from_reader(rdr)
                        .map_err(|e| warn!(?e, ?p, "Unable to parse"))
                        .ok()
                })
                .map(|md: MirrorDefinitions| md.mirrors)
        })
        .unwrap_or_else(|| {
            vec![
                Url::parse("https://download.opensuse.org").unwrap(),
                Url::parse("https://mirrorcache.opensuse.org").unwrap(),
                Url::parse("https://mirrorcache-au.opensuse.org").unwrap(),
                Url::parse("https://mirrorcache-us.opensuse.org").unwrap(),
            ]
        });

    let mut profiled = Vec::with_capacity(mirrors.len());

    for url in mirrors.iter() {
        let r = mirror_latency(url.host_str().unwrap()).await;
        if let Some(lat) = r {
            profiled.push((lat, url))
        }
    }

    profiled.sort_unstable_by(|a, b| a.0.cmp(&b.0).reverse());

    for mp in profiled.iter() {
        debug!("{:?} - {}", mp.0, mp.1.as_str())
    }

    let m = match profiled.pop() {
        Some((l, m)) => {
            info!("Selected - {} - time={:?}", m.as_str(), l);
            m
        }
        None => {
            error!("Mirror profiling failed!");
            std::process::exit(1);
        }
    };

    if !config.doit {
        info!("do it not requested, not changing /etc/zypp/repos.d");
        info!("To update your mirrors re-run with '-x'");
        return;
    }

    // Profile the mirror latencies, since latency is the single
    // largest issues in zypper metadata access.

    let entries = match fs::read_dir("/etc/zypp/repos.d") {
        Ok(e) => e,
        Err(e) => {
            error!(?e, "Unable to read /etc/zypp/repos.d");
            std::process::exit(1);
        }
    };

    let paths: Vec<_> = entries
        .into_iter()
        .filter_map(|ent| ent.ok())
        .map(|ent| ent.path())
        .collect();

    debug!(?paths);

    // Rewrite things.
    paths.iter().for_each(|p| {
        rewrite_mirror(p, &m, &mirrors);
    });

    // wait, if we have files to change, update them.
}
