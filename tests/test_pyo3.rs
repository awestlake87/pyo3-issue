use std::{
    borrow::Cow,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use async_std::{
    net::{TcpListener, ToSocketAddrs},
    task,
};
use async_tungstenite::{async_std::connect_async, tungstenite};
use futures::{channel::oneshot, future, stream::StreamExt};
use lazy_static::lazy_static;
use pyo3::{prelude::*, types::PyTuple};

use pyo3_issue::TEST_SOCKETS;

lazy_static! {
    static ref PYO3_INIT: AtomicBool = AtomicBool::new(true);
}

pub fn init_python() {
    if PYO3_INIT.swap(false, Ordering::Relaxed) {
        // regularly poll for signals
        // - I don't believe this is essential to reproduce the error, but it affected the timing
        // so much that I wasn't able to see the error consistently.
        task::spawn(async move {
            loop {
                Python::with_gil(|py| py.check_signals().map_err(dump_err(py)).unwrap());
                task::sleep(Duration::from_millis(5)).await;
            }
        });
    }
}

fn dump_err<'p>(py: Python<'p>) -> impl FnOnce(PyErr) + 'p {
    move |e| {
        // We can't display Python exceptions via std::fmt::Display,
        // so print the error here manually.
        e.print_and_set_sys_last_vars(py);
    }
}

pub struct PyEventLoop {
    event_loop: PyObject,
    thread: Option<thread::JoinHandle<()>>,
}

impl PyEventLoop {
    pub fn new() -> Self {
        let event_loop = Python::with_gil(|py| {
            let asyncio = py.import("asyncio").map_err(dump_err(py)).unwrap();

            let event_loop = asyncio
                .call0("new_event_loop")
                .map_err(dump_err(py))
                .unwrap();

            PyObject::from(event_loop)
        });

        let event_loop_hdl = event_loop.clone();

        let thread = thread::spawn(move || {
            Python::with_gil(move |py| {
                event_loop_hdl
                    .call_method0(py, "run_forever")
                    .map_err(dump_err(py))
                    .unwrap();

                event_loop_hdl
                    .call_method0(py, "close")
                    .map_err(dump_err(py))
                    .unwrap();
            });
        });

        Self {
            event_loop,
            thread: Some(thread),
        }
    }

    fn call_soon_threadsafe(&self, py: Python, args: impl IntoPy<Py<PyTuple>>) -> PyResult<()> {
        self.event_loop
            .call_method1(py, "call_soon_threadsafe", args)?;

        Ok(())
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            Python::with_gil(|py| {
                log::debug!("join event loop");
                self.call_soon_threadsafe(
                    py,
                    (self
                        .event_loop
                        .getattr(py, "stop")
                        .map_err(dump_err(py))
                        .unwrap(),),
                )
                .map_err(dump_err(py))
                .unwrap();
            });

            thread.join().unwrap();
        }
    }

    pub async fn clean_drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let event_loop = self.event_loop.clone();

            task::spawn_blocking(move || {
                Python::with_gil(|py| {
                    log::debug!("join event loop");
                    event_loop
                        .call_method1(
                            py,
                            "call_soon_threadsafe",
                            (event_loop
                                .getattr(py, "stop")
                                .map_err(dump_err(py))
                                .unwrap(),),
                        )
                        .map_err(dump_err(py))
                        .unwrap();
                });

                thread.join().unwrap();
            })
            .await;
        }
    }
}

impl Drop for PyEventLoop {
    fn drop(&mut self) {
        log::debug!("drop event loop");
        self.join();
    }
}

pub async fn listen(addr: impl ToSocketAddrs, on_bind: oneshot::Sender<()>) {
    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(addr).await;
    let listener = try_socket.expect("Failed to bind");

    on_bind.send(()).unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(async move {
            let mut ws_stream = async_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");

            let mut iso = PyEventLoop::new();

            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(tungstenite::Message::Binary(_)) => (),
                    Ok(tungstenite::Message::Close(frame)) => {
                        log::debug!("close request {:?}", frame);
                    }
                    Ok(msg) => {
                        log::debug!("unhandled message: {:?}", msg);
                    }
                    Err(tungstenite::Error::Protocol(Cow::Borrowed(
                        "Connection reset without closing handshake",
                    ))) => {
                        log::debug!("connection reset without close handshake");
                    }
                    Err(tungstenite::Error::Io(e)) => {
                        log::debug!("isolate connection error: {:?}", e);
                        break;
                    }
                    _ => todo!("{:?}", msg),
                }
            }

            iso.clean_drop().await;
        });
    }
}

fn init_logger() {
    if let Err(_) = env_logger::try_init() {}
}

#[test]
fn test_pyo3_error() {
    init_python();
    init_logger();

    task::block_on(async move {
        // acquire a test port to host the websocket server on
        let addr = ("0.0.0.0", TEST_SOCKETS.lease_port().await.unwrap().port)
            .to_socket_addrs()
            .await
            .unwrap()
            .next()
            .unwrap();

        let host = addr.ip().to_string();
        let port = addr.port();

        // a oneshot that resolves when the server successfully binds to the address
        let (bind_tx, bind_rx) = oneshot::channel();

        // a oneshot that stops the server task when the client is done.
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // spawn the server task in the background
        let server_task = task::spawn(async move {
            // resolve when the task is cancelled or the server loop exits
            future::select(cancel_rx, Box::pin(listen((host.as_str(), port), bind_tx))).await;
        });

        // wait for the server to bind
        bind_rx.await.unwrap();

        // connect to the server
        connect_async(&format!("ws://{}:{}", addr.ip(), addr.port()))
            .await
            .unwrap();

        // act like we're doing something (the setup for the event loop is enough to trigger the error)
        task::sleep(Duration::from_millis(1000)).await;

        // stop the server
        if let Err(_) = cancel_tx.send(()) {
            // already finished
        }

        // join the server task
        server_task.await;
    });
}
