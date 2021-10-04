extern crate insim;
use tokio::time;

#[tokio::main]
pub async fn main() {
    let client = insim::Client::default().using_relay();

    let (shutdown, tx, mut rx) = client.run().await;

    let hlr = insim::packets::Insim::RelayHostListRequest(insim::packets::relay::HostListRequest {
        reqi: 0,
    });

    tx.send(hlr);

    let hs = insim::packets::Insim::RelayHostSelect(insim::packets::relay::HostSelect {
        reqi: 0,

        hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
        admin: "".into(),
        spec: "".into(),
    });

    tx.send(hs);

    // tokio::spawn(async move {
    //     // shutdown after 10s
    //     time::sleep(time::Duration::from_secs(10)).await;
    //     shutdown.send(true);
    // });

    while let Some(event) = rx.recv().await {
        match event {
            Ok(data) => {
                println!("{:?}", data);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}
