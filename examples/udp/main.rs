extern crate insim;

#[tokio::main]
pub async fn main() {
    // TODO: this is out of date. see relay
    let client = insim::Config::default().using_udp("192.168.0.250:29999".into());

    let (shutdown, tx, mut rx) = client.run().await;

    while let Some(packet) = rx.recv().await {
        println!("{:?}", packet);
    }
}
