#![forbid(unsafe_code)]
//! `mercury-cli` — a thin command line over the [`mercury_client`] runtime.
//!
//! Milestone 1 exposes one subcommand, `demo`, which runs a complete two-client
//! end-to-end-encrypted exchange through a running Mercury relay (so you can see the
//! whole inner-ratchet + transport slice work against the real relay binary):
//!
//! ```text
//! # terminal 1
//! cargo run -p mercury-relay --bin mercury-relay-server
//! # terminal 2
//! cargo run -p mercury-client --bin mercury-cli -- demo --relay http://127.0.0.1:8787
//! ```

use std::time::Duration;

use mercury_client::{MercuryClient, RelayTransport, Transport};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("demo") => {
            let relay =
                flag(&args, "--relay").unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
            if let Err(e) = run_demo(&relay) {
                eprintln!("demo failed: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!(
                "mercury-cli — Mercury client runtime demo\n\n\
                 USAGE:\n  mercury-cli demo [--relay URL]\n\n\
                 Runs a two-client end-to-end-encrypted exchange through the relay\n\
                 (default URL http://127.0.0.1:8787). Start the relay first with\n\
                 `cargo run -p mercury-relay --bin mercury-relay-server`."
            );
            std::process::exit(2);
        }
    }
}

/// Read a `--flag VALUE` argument.
fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

/// First 8 bytes of a 32-byte id as hex, for readable logging.
fn short(id: &[u8; 32]) -> String {
    id[..8].iter().map(|b| format!("{b:02x}")).collect()
}

fn run_demo(relay: &str) -> Result<(), String> {
    let transport = RelayTransport::new(relay);
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    println!("relay:  {relay}");
    println!("alice:  {}…", short(&a_id));
    println!("bob:    {}…", short(&b_id));

    // Bob publishes a contact card; Alice opens a session and sends the first flight.
    let card = bob.publish_card();
    let (route, flight) = alice
        .initiate(&card, b"hello over the relay")
        .map_err(|e| format!("initiate: {e}"))?;
    transport
        .submit(&route, &flight)
        .map_err(|e| format!("submit flight: {e}"))?;
    println!(
        "alice -> relay : first flight, {} bytes, route {}…",
        flight.len(),
        short(&route)
    );

    // Bob polls his route, receives, decrypts.
    let blob = poll_until(&transport, &bob)?;
    let (_from, msg) = bob
        .receive(&blob)
        .map_err(|e| format!("bob receive: {e}"))?;
    println!("bob   <- relay : \"{}\"", String::from_utf8_lossy(&msg));

    // Bob replies through the relay.
    let reply = bob
        .send(&a_id, b"hi alice, got it")
        .map_err(|e| format!("bob send: {e}"))?;
    transport
        .submit(&a_id, &reply)
        .map_err(|e| format!("submit reply: {e}"))?;
    let blob = poll_until(&transport, &alice)?;
    let (_from, msg) = alice
        .receive(&blob)
        .map_err(|e| format!("alice receive: {e}"))?;
    println!("alice <- relay : \"{}\"", String::from_utf8_lossy(&msg));

    println!("\nOK: end-to-end-encrypted exchange through the relay succeeded.");
    Ok(())
}

fn poll_until(transport: &RelayTransport, client: &MercuryClient) -> Result<Vec<u8>, String> {
    let route = client.account_id();
    // Prove route ownership so the relay authorizes draining our queue. One fresh proof covers
    // the short poll loop (well within the relay's freshness window).
    let auth = client.sign_poll_auth(now_unix_s());
    for _ in 0..100 {
        match transport.poll(&route, &auth) {
            Ok(Some(blob)) => return Ok(blob),
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(format!("poll: {e}")),
        }
    }
    Err("no message delivered within the timeout".to_string())
}

fn now_unix_s() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
