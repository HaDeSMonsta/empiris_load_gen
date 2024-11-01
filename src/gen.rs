use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::comm;
use rand::rngs::StdRng;
use rand::Rng;
use tracing::debug;

pub fn go(
    addr: SocketAddr,
    mut rng: StdRng,
    rx: Receiver<()>,
    results: Arc<Mutex<Vec<Duration>>>,
    id: u16,
) {
    let mut local_times = Vec::new();
    loop {
        if let Ok(_) = rx.try_recv() {
            debug!("Thread [{id}]: Received signal, attempting to get result lock");
            {
                debug!("Thread [{id}]: Got result lock");
                let mut results = results.lock().unwrap();
                results.extend(local_times);
                    debug!("Thread [{id}]: Wrote results, dropping lock and stopping");
            }
            break;
        }
        debug!("Thread [{id}]: Running");

        let x = rng.gen_range(0..(i32::MAX / 2));
        let y = rng.gen_range(0..(i32::MAX / 2));
        let operation = rng.gen_range(0..=3);

        debug!("Thread [{id}]: Created task");

        let expected = match operation {
            0 => x + y,
            1 => x - y,
            2 => x / y,
            3 => x % y,
            _ => panic!("Impossible state"),
        };
        debug!("Thread [{id}]: Created expected");

        let start = Instant::now();
        debug!("Thread [{id}]: Got start time");
        let res = comm::send(addr, x, y, operation, id);
        debug!("Thread [{id}]: Got result");
        let Some(res) = res else {
            let sleep_time = 10;
            debug!("Thread [{id}]: Res is None, sleeping for {sleep_time} ms");
            sleep(Duration::from_millis(sleep_time));
            continue;
        };
        debug!("Thread [{id}]: Res is Some");
        let elapsed = start.elapsed();
        debug!("Thread [{id}]: Got elapsed time");
        local_times.push(elapsed);
        debug!("Thread [{id}]: Pushed time");

        assert_eq!(expected, res, "Unexpected response");
        debug!("Thread [{id}]: Asserted response");
    }
}

