use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use crate::comm;
use crate::math::MathTask;
use logger_utc::log;
use rand::rngs::StdRng;
use rand::Rng;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::time::Instant;

pub async fn go(
    addr: SocketAddr,
    mut rng: StdRng,
    mut rx: Receiver<()>,
    results: Arc<Mutex<Vec<Duration>>>,
    id: u16,
) {
    let mut local_times = Vec::new();
    loop {
        if let Ok(_) = rx.try_recv() {
            log(format!("Thread [{id}]: Received signal, attempting to get result lock"));
            {
                log(format!("Thread [{id}]: Got result lock"));
                let mut results = results.lock().await;
                results.extend(local_times);
                log(format!("Thread [{id}]: Wrote results, dropping lock and stopping"));
            }
            break;
        }

        let x = rng.gen_range(0..(i32::MAX / 2));
        let y = rng.gen_range(0..(i32::MAX / 2));
        let operation = rng.gen_range(0..=3);

        let task = MathTask {
            x,
            y,
            operation,
        };

        let expected = match operation {
            0 => x + y,
            1 => x - y,
            2 => x / y,
            3 => x % y,
            _ => panic!("Impossible state"),
        };

        let start = Instant::now();
        let res = comm::send(addr, task).await;
        let Some(res) = res else {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        };
        let elapsed = start.elapsed();
        local_times.push(elapsed);

        assert_eq!(expected, res.sol, "Unexpected response");
    }
}
