extern crate insim;

#[tokio::main]
pub async fn main() {
    let client = insim::Client::new_udp("insim.rs".to_string(), "192.168.0.250:29999".to_string());

    let (shutdown, tx, mut rx) = client.run().await;

    while let Some(packet) = rx.recv().await {
        println!("{:?}", packet);
    }
}
