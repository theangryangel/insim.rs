//! Decode a raw insim packet from hex bytes, printing a field-level trace of the decode process.
//!
//! The input is the full on-wire packet bytes, including the leading size byte:
//!   - Byte 0: packet type discriminant
//!   - Byte 1: reqi
//!   - Byte 2+: packet payload
//!
//! Examples:
//!   cargo run -p decode-trace -- 030203
//!   cargo run -p decode-trace -- '\x01\x03\x02\x03'
use bytes::Bytes;
use clap::Parser;
use insim::core::{Decode, DecodeContext};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_tree::HierarchicalLayer;

#[derive(Parser)]
#[command(author, version, about)]
/// Decode a raw insim packet and print a field-level trace.
///
/// Accepts hex bytes or C-style escape sequences:
///   decode-trace 01 03 02 03
///   decode-trace 01030203
///   decode-trace '\x01\x03\x02\x03'
struct Cli {
    /// Bytes of the full on-wire packet (size byte first), as hex or escape sequences
    bytes: Vec<String>,
}

/// Parse a string containing C-style escape sequences into bytes.
/// Handles \xNN (hex), \0 (null), \n, \r, \t, and \\ (literal backslash).
fn parse_escape_sequences(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            // literal byte - only accept printable ASCII here
            if c.is_ascii() {
                out.push(c as u8);
            } else {
                return Err(format!("unexpected non-ASCII character: {c:?}"));
            }
            continue;
        }
        match chars.next() {
            Some('x') => {
                let hi = chars.next().ok_or("truncated \\x escape")?;
                let lo = chars.next().ok_or("truncated \\x escape")?;
                let hex = format!("{hi}{lo}");
                let byte = u8::from_str_radix(&hex, 16)
                    .map_err(|_| format!("invalid hex in \\x escape: {hex:?}"))?;
                out.push(byte);
            },
            Some('0') => out.push(0),
            Some('n') => out.push(b'\n'),
            Some('r') => out.push(b'\r'),
            Some('t') => out.push(b'\t'),
            Some('\\') => out.push(b'\\'),
            Some(other) => return Err(format!("unknown escape: \\{other}")),
            None => return Err("trailing backslash".into()),
        }
    }
    Ok(out)
}

fn main() {
    // Force insim_core to TRACE so every field decode is logged.
    // Override specific targets via RUST_LOG if needed.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::WARN.into())
                .from_env_lossy()
                .add_directive("insim_core=trace".parse().expect("valid directive")),
        )
        .with(HierarchicalLayer::new(2).with_targets(true))
        .init();

    let cli = Cli::parse();

    let joined = cli.bytes.join("");

    // Detect format: if the input contains a backslash, treat it as escape sequences.
    // Otherwise treat it as run-together hex digits.
    let raw: Vec<u8> = if joined.contains('\\') {
        match parse_escape_sequences(&joined) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            },
        }
    } else {
        let hex_input: String = joined.chars().filter(|c| c.is_ascii_hexdigit()).collect();

        if hex_input.len() % 2 != 0 {
            eprintln!("error: odd number of hex digits");
            std::process::exit(1);
        }

        hex_input
            .as_bytes()
            .chunks(2)
            .map(|pair| {
                let s = std::str::from_utf8(pair).expect("ascii");
                u8::from_str_radix(s, 16).expect("valid hex digit")
            })
            .collect()
    };

    if raw.is_empty() {
        eprintln!("error: no bytes provided");
        std::process::exit(1);
    }

    eprintln!("--- input ({} bytes): {:02x?}", raw.len(), raw);
    eprintln!("--- decode trace:");

    // Skip the framing size byte (byte 0) - it isn't part of the Packet encoding.
    // Packet::decode expects: [type_byte, reqi, ...payload...]
    let mut buf = Bytes::copy_from_slice(&raw[1..]);
    let mut ctx = DecodeContext::new(&mut buf);

    match insim::Packet::decode(&mut ctx) {
        Ok(packet) => {
            eprintln!("--- result: {packet:#?}");
        },
        Err(e) => {
            eprintln!("--- result: decode error: {e}");
        },
    }
}
