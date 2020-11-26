use std::{
    collections::HashSet,
    os::unix::io::AsRawFd,
    sync::{Arc, Mutex},
};

use async_std::net::TcpListener;
use lazy_static::lazy_static;
use nix::sys::socket::{self, sockopt::ReusePort};

lazy_static! {
    pub static ref TEST_SOCKETS: TestSocketManager = TestSocketManager::new();
}

pub struct PortLease {
    pub port: u16,
}

impl Drop for PortLease {
    fn drop(&mut self) {
        TEST_SOCKETS.drop_port(self.port);
    }
}

pub struct TestSocketManager {
    next_port: Arc<Mutex<u16>>,
    ports: Arc<Mutex<HashSet<u16>>>,
}

impl TestSocketManager {
    pub fn new() -> Self {
        Self {
            next_port: Arc::new(Mutex::new(8000)),
            ports: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn lease_port(&self) -> Option<PortLease> {
        for _ in 8000..9000 {
            let port = {
                let mut next_port = self.next_port.lock().unwrap();
                let port = *next_port;

                *next_port += 1;
                if *next_port >= 9000 {
                    *next_port = 8000;
                }

                port
            };

            {
                let mut ports = self.ports.lock().unwrap();

                if ports.contains(&port) {
                    continue;
                } else {
                    ports.insert(port);
                }
            }

            // Check if the specified port if available.
            match TcpListener::bind(("0.0.0.0", port)).await {
                Ok(listener) => {
                    socket::setsockopt(listener.as_raw_fd(), ReusePort, &true).unwrap();
                    return Some(PortLease { port });
                }
                Err(_) => {
                    self.ports.lock().unwrap().remove(&port);

                    continue;
                }
            }
        }

        None
    }

    pub fn drop_port(&self, port: u16) {
        self.ports.lock().unwrap().remove(&port);
    }
}
