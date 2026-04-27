//! PyO3 FFI layer for pyinsim.
//!
//! **Architecture:** The boundary between Rust and Python is JSON strings only.
//! No PyO3 lifetime/newtype hell — every method either returns or accepts a
//! `String` containing `serde_json`-serialized [`insim::Packet`] data.
//!
//! The [`_Insim`] class is private to the Python package; the public facade
//! lives in `pyinsim/client.py`.

#![allow(missing_docs)] // User-facing docs live in .pyi stubs and the Python package.
#![allow(missing_debug_implementations)] // _Insim fields lack Debug in some tokio versions.

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use tokio::{runtime::Builder as RuntimeBuilder, sync::broadcast::error::RecvError};

// ── FFI class ──────────────────────────────────────────────────────────────

/// Raw FFI handle wrapping a spawned insim connection.
///
/// Do not use this directly from Python — the `InsimClient` facade in
/// `pyinsim.client` is the public API.
#[pyclass]
struct _Insim {
    runtime: tokio::runtime::Runtime,
    task: insim::builder::InsimTask,
    receiver: tokio::sync::broadcast::Receiver<insim::Packet>,
    // Kept alive so the background network task is not silently detached.
    _handle: tokio::task::JoinHandle<insim::Result<()>>,
}

#[pymethods]
impl _Insim {
    /// Connect to an LFS InSim endpoint and return a ready handle.
    ///
    /// `addr` – TCP address string, e.g. `"127.0.0.1:29999"`.
    ///
    /// All remaining arguments are keyword-only:
    /// - `flags` – list of `IsiFlag` enum values from `_types` (default: none)
    /// - `iname` – InSim application name shown in LFS (default: `"pyinsim"`)
    /// - `admin_password` – host admin password if required (default: none)
    /// - `interval_ms` – NLP/MCI update interval in milliseconds (default: LFS default)
    /// - `prefix` – single-character command prefix (default: none)
    /// - `capacity` – broadcast channel buffer size (default: 128)
    #[staticmethod]
    #[pyo3(signature = (addr, *, flags=None, iname=None, admin_password=None, interval_ms=None, prefix=None, capacity=128))]
    fn connect(
        addr: &str,
        flags: Option<Vec<String>>,
        iname: Option<String>,
        admin_password: Option<String>,
        interval_ms: Option<u64>,
        prefix: Option<String>,
        capacity: usize,
    ) -> PyResult<Self> {
        let runtime = RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

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

        let (task, handle) = runtime
            .block_on(
                insim::tcp(addr)
                    .isi_flags(isi_flags)
                    .isi_iname(iname.unwrap_or_else(|| "pyinsim".to_string()))
                    .isi_admin_password(admin_password)
                    .isi_interval(interval_ms.map(std::time::Duration::from_millis))
                    .isi_prefix(prefix_char)
                    .spawn(capacity),
            )
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let receiver = task.subscribe();

        Ok(Self {
            runtime,
            task,
            receiver,
            _handle: handle,
        })
    }

    /// Block (releasing the GIL) until a packet arrives, returning it as JSON.
    ///
    /// The returned string always contains a `"type"` key with the Rust enum
    /// variant name, e.g. `{"type":"Ncn","ucid":1,...}`.
    ///
    /// Lagged messages (channel overflow) are silently skipped.
    /// Raises `RuntimeError` when the connection is closed.
    fn recv(&mut self, py: Python<'_>) -> PyResult<String> {
        let receiver = &mut self.receiver;
        let rt = &self.runtime;

        // `detach` is PyO3 0.22+'s replacement for `allow_threads`.
        // It releases the GIL for the duration of the closure so the Python
        // event loop keeps ticking while we block on the broadcast channel.
        py.detach(|| {
            rt.block_on(async {
                loop {
                    match receiver.recv().await {
                        Ok(packet) => {
                            return serde_json::to_string(&packet)
                                .map_err(|e| PyRuntimeError::new_err(e.to_string()));
                        },
                        // Channel overflowed — skip the gap and keep reading.
                        Err(RecvError::Lagged(_)) => continue,
                        Err(e) => {
                            return Err(PyRuntimeError::new_err(e.to_string()));
                        },
                    }
                }
            })
        })
    }

    /// Send a packet to LFS. `data` must be a JSON string with a `"type"` field
    /// matching a Rust `Packet` variant name, e.g.
    /// `{"type":"Tiny","reqi":1,"subt":"Ping"}`.
    ///
    /// Raises `ValueError` for malformed JSON and `RuntimeError` if the
    /// connection is dead.
    fn send(&self, py: Python<'_>, data: String) -> PyResult<()> {
        let packet: insim::Packet = serde_json::from_str(&data)
            .map_err(|e| PyValueError::new_err(format!("Invalid packet JSON: {e}")))?;

        let task = self.task.clone();
        let rt = &self.runtime;

        py.detach(|| {
            rt.block_on(task.send(packet))
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Signal the background network actor to stop gracefully.
    fn shutdown(&self, py: Python<'_>) {
        let task = self.task.clone();
        let rt = &self.runtime;
        py.detach(|| rt.block_on(task.shutdown()));
    }
}

// ── Module entry point ─────────────────────────────────────────────────────

/// Register the `_Insim` class into the `pyinsim._insim` extension module.
#[pymodule]
fn _insim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = pyo3_log::init();
    m.add_class::<_Insim>()?;
    Ok(())
}

// ── Unit tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // These tests verify the exact JSON shape that the Python dispatcher expects:
    // a `"type"` key containing the Rust enum variant name, plus all struct
    // fields at the top level (serde's "internally tagged" format).

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
