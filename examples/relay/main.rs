extern crate insim;

#[tokio::main]
pub async fn main() {
    let mut client = insim::Client::new_relay("insim.rs".to_string()).await;

    let hlr = insim::packets::Insim::RelayHostListRequest(insim::packets::relay::HostListRequest {
        reqi: 0,
    });

    client.send(hlr).await;

    let hs = insim::packets::Insim::RelayHostSelect(insim::packets::relay::HostSelect {
        reqi: 0,

        hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
        admin: "".into(),
        spec: "".into(),
    });

    client.send(hs).await;

    client.run().await;

    /*
    while let Some(result) = client.recv().await {
        match result {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
    */
}
