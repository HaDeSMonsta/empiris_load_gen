use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use logger_utc::log;
use rand::rngs::StdRng;

pub fn go(addr: SocketAddr, rng: StdRng, rx: Receiver<()>, results: Arc<Mutex<Vec<u8>>>, id: u8) {
    loop {
        if let Ok(_) = rx.try_recv() {
            log(format!("Thread [{id}]: Received signal, attempting to get result lock"));
            {
                log(format!("Thread [{id}]: Got result lock"));
                let mut results = results.lock().unwrap();
                results.push(id);
                log(format!("Thread [{id}]: Wrote results, dropping lock and stopping"));
            }
            break
        }

        thread::sleep(Duration::from_secs(3));
    }
}
