use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::env;
use std::net::SocketAddr;

/// Generated via rand itself
const DEFAULT_SEED: [u8; 32] = [135, 142, 87, 161, 18, 25, 215, 9, 131, 174, 29, 172, 100, 27, 29, 209, 74, 97, 60, 11, 34, 210, 34, 123, 121, 140, 210, 228, 230, 164, 1, 28];

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
    /// Seed to use (please don't use arg for that, use .env file)
    #[arg(short, long, use_value_delimiter = true)]
    seed: Option<String>,
}

fn get_args() -> (SocketAddr, [u8; 32]) {
    let args = Args::parse();
    let _ = dotenv::dotenv();

    let ip = if let Some(ip) = args.target_ip {
        ip
    } else {
        handle_env("TARGET_IP")
    };

    let port = if let Some(port) = args.target_port {
        port
    } else {
        handle_env("TARGET_PORT")
            .parse()
            .expect("Port must be a valid u16")
    };

    let tmp_seed = if let Some(seed) = args.seed {
        Some(seed)
    } else if let Ok(seed) = env::var("") {
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

    let addr = format!("{ip}:{port}");
    (
        addr.parse().expect(&format!("{addr} is not a valid SocketAddr")),
        seed,
    )
}

fn handle_env(key: &'static str) -> String {
    env::var(key)
        .expect(&format!("If not provided as arg, {key} must be set in the .env file"))
}

fn main() {
    println!("Hello, world!");

    let (addr, seed) = get_args();
    println!("Addr: {addr}\nSeed: {seed:?}\n");

    let mut rng = StdRng::from_seed(seed);
    println!("Rngd: {}", rng.gen::<u8>());
}
