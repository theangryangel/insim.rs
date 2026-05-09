//! PyO3 FFI layer for insim-o3.
//!
//! The boundary between Rust and Python is JSON strings only.
//! No PyO3 lifetime/newtype hell - every method either returns or accepts a
//! `String` containing `serde_json`-serialized [`insim::Packet`] data.
//!
//! Every blocking operation (`connect`, `recv`, `send`, `shutdown`) returns a
//! Python awaitable via `pyo3-async-runtimes`. The library is async-only; the
//! public Python facade in `insim-o3/client.py` exposes it as `async with
//! Insim(...) as client:`.

#![allow(missing_docs)] // User-facing docs live in .pyi stubs and the Python package.
#![allow(missing_debug_implementations)] // _Insim fields lack Debug in some tokio versions.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use tokio::{
    runtime::Runtime,
    sync::{Notify, broadcast::error::RecvError},
};

/// Shared Tokio runtime managed by `pyo3-async-runtimes`.
///
/// Configured once in [`_insim`]'s module init via
/// [`pyo3_async_runtimes::tokio::init`].
fn runtime() -> &'static Runtime {
    pyo3_async_runtimes::tokio::get_runtime()
}

/// Do not use this directly from Python - the `Insim` facade in
/// `insim-o3.client` is the public API.
#[pyclass]
struct _Insim {
    task: insim::builder::InsimTask,
    /// Wrapped in `Arc<Mutex>` so each `recv` future can move a clone into the
    /// `'static` Tokio future returned to asyncio.
    receiver: Arc<tokio::sync::Mutex<tokio::sync::broadcast::Receiver<insim::Packet>>>,
    /// Set by the watcher task once the network actor exits. Read inside the
    /// recv loop to handle the race where the watcher fires `notify_waiters`
    /// before any `recv` future is awaiting.
    is_done: Arc<AtomicBool>,
    /// Fired by the watcher task to wake any in-flight `recv` immediately.
    shutdown_signal: Arc<Notify>,
    /// Pre-formatted exit message, populated before `is_done` is set.
    exit_msg: Arc<Mutex<Option<String>>>,
}

#[pymethods]
impl _Insim {
    /// Connect to an LFS InSim endpoint. Returns an awaitable resolving to a
    /// ready `_Insim`.
    #[staticmethod]
    #[pyo3(signature = (addr, *, flags=None, iname=None, admin_password=None, interval_ms=None, prefix=None, capacity=512))]
    fn connect<'py>(
        py: Python<'py>,
        addr: String,
        flags: Option<Vec<String>>,
        iname: Option<String>,
        admin_password: Option<String>,
        interval_ms: Option<u64>,
        prefix: Option<String>,
        capacity: usize,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Validate sync-friendly args before going async so `ValueError` is
        // raised from `connect()` itself, not from the awaitable.
        let isi_flags = match flags {
            None => insim::insim::IsiFlags::empty(),
            Some(names) => {
                serde_json::from_value::<insim::insim::IsiFlags>(serde_json::Value::Array(
                    names.into_iter().map(serde_json::Value::String).collect(),
                ))
                .map_err(|e| PyValueError::new_err(format!("Invalid IsiFlag: {e}")))?
            },
        };

        let prefix_char = match prefix.as_deref() {
            None | Some("") => None,
            Some(s) => {
                let mut chars = s.chars();
                let c = chars.next().unwrap();
                if chars.next().is_some() {
                    return Err(PyValueError::new_err("prefix must be a single character"));
                }
                Some(c)
            },
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (task, handle) = insim::tcp(addr.as_str())
                .isi_flags(isi_flags)
                .isi_iname(iname.unwrap_or_else(|| "insim-o3".to_owned()))
                .isi_admin_password(admin_password)
                .isi_interval(interval_ms.map(std::time::Duration::from_millis))
                .isi_prefix(prefix_char)
                .spawn(capacity)
                .await
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

            let receiver = Arc::new(tokio::sync::Mutex::new(task.subscribe()));
            let is_done = Arc::new(AtomicBool::new(false));
            let shutdown_signal = Arc::new(Notify::new());
            let exit_msg = Arc::new(Mutex::new(None));

            {
                let is_done = is_done.clone();
                let shutdown_signal = shutdown_signal.clone();
                let exit_msg = exit_msg.clone();
                let _watcher = runtime().spawn(async move {
                    let msg = match handle.await {
                        Ok(Ok(())) => "connection closed".to_owned(),
                        Ok(Err(e)) => e.to_string(),
                        Err(e) => format!("background task panicked: {e}"),
                    };
                    if let Ok(mut guard) = exit_msg.lock() {
                        *guard = Some(msg);
                    }
                    is_done.store(true, Ordering::Release);
                    shutdown_signal.notify_waiters();
                });
            }

            Ok(Self {
                task,
                receiver,
                is_done,
                shutdown_signal,
                exit_msg,
            })
        })
    }

    /// Await the next packet from LFS, returning it as a JSON string.
    ///
    /// The returned string always contains a `"type"` key with the Rust enum
    /// variant name, e.g. `{"type":"Ncn","ucid":1,...}`.
    ///
    /// Lagged messages (channel overflow) are silently skipped.
    /// Raises `RuntimeError` when the connection is closed.
    fn recv<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let receiver = self.receiver.clone();
        let shutdown = self.shutdown_signal.clone();
        let is_done = self.is_done.clone();
        let exit_msg = self.exit_msg.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            loop {
                if is_done.load(Ordering::Acquire) {
                    return Err(disconnect_error(&exit_msg));
                }
                let outcome = {
                    let mut rx = receiver.lock().await;
                    tokio::select! {
                        biased;
                        () = shutdown.notified() => RecvOutcome::Disconnected,
                        res = rx.recv() => RecvOutcome::Packet(res),
                    }
                };
                match outcome {
                    RecvOutcome::Packet(Ok(packet)) => {
                        return serde_json::to_string(&packet)
                            .map_err(|e| PyRuntimeError::new_err(e.to_string()));
                    },
                    RecvOutcome::Packet(Err(RecvError::Lagged(_))) => continue,
                    RecvOutcome::Packet(Err(_)) | RecvOutcome::Disconnected => {
                        return Err(disconnect_error(&exit_msg));
                    },
                }
            }
        })
    }

    /// Send a packet to LFS. `data` must be a JSON string with a `"type"` field
    /// matching a Rust `Packet` variant name, e.g.
    /// `{"type":"Tiny","reqi":1,"subt":"Ping"}`.
    ///
    /// Raises `ValueError` for malformed JSON and `RuntimeError` if the
    /// connection is dead.
    fn send<'py>(&self, py: Python<'py>, data: &str) -> PyResult<Bound<'py, PyAny>> {
        let packet = parse_packet(data)?;
        let task = self.task.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            task.send(packet)
                .await
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Signal the background network actor to stop gracefully.
    fn shutdown<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let task = self.task.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            task.shutdown().await;
            Ok(())
        })
    }
}

fn parse_packet(data: &str) -> PyResult<insim::Packet> {
    serde_json::from_str(data)
        .map_err(|e| PyValueError::new_err(format!("Invalid packet JSON: {e}")))
}

fn disconnect_error(exit_msg: &Mutex<Option<String>>) -> PyErr {
    let msg = exit_msg
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| "connection closed".to_owned());
    PyRuntimeError::new_err(msg)
}

impl Drop for _Insim {
    fn drop(&mut self) {
        // Best-effort: tell the network actor to stop so its socket closes.
        // The watcher task will then complete and free its captured Arcs.
        let task = self.task.clone();
        let _shutdown = runtime().spawn(async move { task.shutdown().await });
    }
}

enum RecvOutcome {
    Packet(Result<insim::Packet, RecvError>),
    Disconnected,
}

/// Strip LFS colour markers (`^0`–`^8`) from a string.
///
/// Escaped control markers (`^^`) are preserved so they survive a subsequent
/// call to :func:`unescape`.  Call this before :func:`unescape` when you need
/// both operations.
#[pyfunction]
fn strip_colours(input: &str) -> String {
    insim::core::string::colours::strip(input).into_owned()
}

/// Unescape LFS escape sequences (e.g. ``^v`` → ``|``, ``^t`` → ``"``).
///
/// If you also need to strip colours, call :func:`strip_colours` first while
/// the marker intent is still preserved.
#[pyfunction]
fn unescape(input: &str) -> String {
    insim::core::string::escaping::unescape(input).into_owned()
}

/// Escape a string for sending to LFS (e.g. ``|`` → ``^v``, ``"`` → ``^t``).
#[pyfunction]
fn escape(input: &str) -> String {
    insim::core::string::escaping::escape(input).into_owned()
}

/// Split an LFS string into ``(colour_index, text)`` spans.
///
/// Colour index is ``0``–``8`` (matching LFS ``^0``–``^8``).  Spans with no
/// text are skipped.  The text slices may still contain escaped sequences
/// (``^^``); call :func:`unescape` on each span if you need the final display
/// string.
#[pyfunction]
fn colour_spans(input: &str) -> Vec<(u8, String)> {
    insim::core::string::colours::spans(input)
        .map(|(c, s)| (c, s.to_owned()))
        .collect()
}

/// Register the `_Insim` class into the `insim-o3._insim` extension module.
#[pymodule]
fn _insim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = pyo3_log::init();

    // Configure the shared Tokio runtime used by every `future_into_py` call.
    // Must be set before the first `pyo3_async_runtimes::tokio::get_runtime()`.
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    let _ = builder.enable_all().thread_name("insim-o3");
    pyo3_async_runtimes::tokio::init(builder);

    m.add_class::<_Insim>()?;
    m.add_function(wrap_pyfunction!(strip_colours, m)?)?;
    m.add_function(wrap_pyfunction!(unescape, m)?)?;
    m.add_function(wrap_pyfunction!(escape, m)?)?;
    m.add_function(wrap_pyfunction!(colour_spans, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use insim::Packet;

    #[test]
    fn ncn_serializes_with_type_tag() {
        let packet = Packet::Ncn(insim::insim::Ncn {
            ucid: insim::identifiers::ConnectionId(7),
            uname: "racer".to_owned(),
            pname: "Speedy".to_owned(),
            admin: true,
            total: 3,
            ..Default::default()
        });

        let json = serde_json::to_string(&packet).expect("serialize Ncn");
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");

        assert_eq!(v["type"], "Ncn", "missing or wrong type discriminator");
        assert_eq!(v["ucid"], 7);
        assert_eq!(v["uname"], "racer");
        assert_eq!(v["pname"], "Speedy");
        assert_eq!(v["admin"], true);
        assert_eq!(v["total"], 3);
        // flags serializes as a JSON array (bitflags serde module)
        assert!(v["flags"].is_array());
    }

    #[test]
    fn mso_serializes_with_type_tag() {
        let packet = Packet::Mso(insim::insim::Mso {
            ucid: insim::identifiers::ConnectionId(2),
            msg: "hello world".to_owned(),
            ..Default::default()
        });

        let json = serde_json::to_string(&packet).expect("serialize Mso");
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");

        assert_eq!(v["type"], "Mso");
        assert_eq!(v["ucid"], 2);
        assert_eq!(v["msg"], "hello world");
        // MsoUserType serializes as the variant name string
        assert_eq!(v["usertype"], "System");
    }

    #[test]
    fn tiny_round_trips_through_json() {
        let original = Packet::Tiny(insim::insim::Tiny {
            reqi: insim::identifiers::RequestId(1),
            subt: insim::insim::TinyType::Ncn,
        });

        let json = serde_json::to_string(&original).expect("serialize Tiny");
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse JSON");
        assert_eq!(v["type"], "Tiny");
        assert_eq!(v["reqi"], 1);
        // TinyType enum serializes as the variant name string
        assert_eq!(v["subt"], "Ncn");

        // Full round-trip: JSON → Packet → JSON must produce identical output.
        let roundtripped: Packet = serde_json::from_str(&json).expect("deserialize Tiny");
        let json2 = serde_json::to_string(&roundtripped).expect("re-serialize Tiny");
        assert_eq!(json, json2, "Tiny round-trip produced different JSON");
    }
}
