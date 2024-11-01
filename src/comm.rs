use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize)]
struct MathTask {
    x: i32,
    y: i32,
    operation: u8,
}

#[derive(Deserialize)]
struct MathRes {
    res: i32,
}

pub fn send(addr: SocketAddr, x: i32, y: i32, operation: u8, id: u16) -> Option<i32> {
    debug!("Thread [{id}]: Called send");
    let task = MathTask {
        x,
        y,
        operation,
    };
    let math_str = serde_json::to_string(&task).unwrap();
    debug!("Thread [{id}]: Created task and serialized: {math_str}");
    let math_str = format!("{math_str}\n");

    let Ok(mut stream) = TcpStream::connect(addr) else {
        debug!("Thread [{id}]: Unable to connect to server");
        return None;
    };
    debug!("Thread [{id}]: Connected stream");

    stream.set_nonblocking(true).unwrap();
    debug!("Thread [{id}]: Is nonblocking");

    match stream.write_all(math_str.as_bytes()) {
        Ok(_) => debug!("Thread [{id}]: Wrote to stream"),
        Err(_) => {
            debug!("Thread [{id}]: Unable to write to stream");
            return None;
        }
    };

    let start = Instant::now();
    let mut tmp_buf = [0; 1024];
    let mut buf = vec![];
    'outer: loop {
        if start.elapsed() > Duration::from_secs(5) {
            debug!("Thread [{id}]: Timeout reached");
            return None;
        }

        match stream.read(&mut tmp_buf) {
            Ok(0) => {
                debug!("Thread [{id}]: read from server terminated");
                return None;
            }
            Ok(size) => {
                for byte in &tmp_buf[0..size] {
                    if *byte == b'\n' {
                        debug!("Thread [{id}]: Reached end of response");
                        break 'outer;
                    }
                    buf.push(*byte);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(_) => {
                debug!("Thread [{id}]: Unable to read from stream");
                return None;
            }
        }
    }

    let res = String::from_utf8_lossy(&buf).to_string();
    debug!("Thread [{id}]: Response {res}");
    let res = serde_json::from_str::<MathRes>(&res).unwrap();
    debug!("Thread [{id}]: Converted to struct");

    Some(res.res)
}

