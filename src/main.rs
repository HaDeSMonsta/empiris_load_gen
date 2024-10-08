mod comm;
mod gen;

use crate::gen::go;
use clap::Parser;
use rand::{Rng, SeedableRng};
use std::any::type_name;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread};
use logger_utc::log;
use rand::rngs::StdRng;

/// Generated via rand itself
const DEFAULT_SEED: [u8; 32] = [
    135,
    142,
    87,
    161,
    18,
    25,
    215,
    9,
    131,
    174,
    29,
    172,
    100,
    27,
    29,
    209,
    74,
    97,
    60,
    11,
    34,
    210,
    34,
    123,
    121,
    140,
    210,
    228,
    230,
    164,
    1,
    28
];

/// Load generator for mock SUT
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The target IP Address
    #[arg(long)]
    target_ip: Option<String>,

    /// The target port
    #[arg(long)]
    target_port: Option<u16>,

    /// Number of worker threads
    #[arg(short, long)]
    num_threads: Option<u8>,

    /// Seconds to wait until termination
    #[arg(long)]
    time_secs: Option<u64>,

    /// Seed to use (please don't use args for that, use .env file)
    #[arg(short, long, use_value_delimiter = true)]
    seed: Option<String>,
}

fn get_args() -> (SocketAddr, u8, u64, [u8; 32]) {
    let args = Args::parse();
    let _ = dotenv::dotenv();

    let ip = if let Some(ip) = args.target_ip {
        ip
    } else {
        handle_env("TARGET_IP")
    };

    let port = args.target_port
                   .unwrap_or(handle_env("TARGET_PORT"));

    let addr = format!("{ip}:{port}");

    let tmp_seed = if let Some(seed) = args.seed {
        Some(seed)
    } else if let Ok(seed) = env::var("SEED") {
        Some(seed)
    } else {
        None
    };

    let seed = if let Some(seed) = tmp_seed {
        let seed_vec = seed.split(",")
                           .map(|s| s.trim())
                           .map(|s| s.parse::<u8>().expect("Seed must only contain u8s"))
                           .collect::<Vec<_>>();
        assert_eq!(32, seed_vec.len(), "Seed must be 32 u8s");

        let mut seed = [0; 32];
        for i in 0..32 {
            seed[i] = seed_vec[i];
        }
        seed
    } else {
        DEFAULT_SEED
    };

    let num_threads = args.num_threads
                          .unwrap_or(handle_env("NUM_THREADS"));
    assert_ne!(0, num_threads, "How do you want to run with 0 threads?");

    let time_secs = args.time_secs
                        .unwrap_or(handle_env("TIME_SECS"));
    (
        addr.parse().expect(&format!("{addr} is not a valid SocketAddr")),
        num_threads,
        time_secs,
        seed,
    )
}

fn handle_env<T: FromStr>(key: &'static str) -> T
where
    <T as FromStr>::Err: Debug,
{
    env::var(key)
        .expect(&format!("If not provided as arg, {key} must be set in the .env file"))
        .parse()
        .expect(&format!("Unable to parse variable {key} to {}", type_name::<T>()))
}

fn main() {
    let (addr, num_threads, sleep_secs, seed) = get_args();
    let mut rng = StdRng::from_seed(seed);

    let mut channels = Vec::new();
    let mut workers = Vec::new();
    let results = Arc::new(Mutex::new(Vec::new()));

    log(format!("Configuration is ok, starting {num_threads} threads"));

    for id in 0..num_threads {
        let (tx, rx) = mpsc::channel::<()>();
        let rng = StdRng::from_seed(rng.gen());
        let results = results.clone();

        let handle = thread::spawn(move ||
            go(addr, rng, rx, results, id)
        );

        channels.push(tx);
        workers.push(handle);
    }

    log(format!("We are running, sleeping {sleep_secs} seconds"));

    sleep(Duration::from_secs(sleep_secs));

    log("Stopping threads");

    for channel in channels {
        channel.send(()).unwrap()
    }

    log("Send stop signal, joining");

    for worker in workers {
        worker.join().unwrap();
    }

    {
        let results = results.lock();
        log(format!("Results: {results:?}"));
    }

    log("Joined all workers, shutting down");
}
