extern crate insim;

#[tokio::main]
pub async fn main() {
    let mut client =
        insim::Client::connect("insim.rs".to_string(), "isrelay.lfs.net:47474".to_string()).await;

    let hlr = insim::Packets::RelayHostListRequest { reqi: 0 };

    client.send(hlr).await;

    let hs = insim::Packets::RelaySelect {
        reqi: 0,

        hname: "^0[^7MR^0c] ^7Beginner ^0BMW".to_string(),
        admin: "".to_string(),
        spec: "".to_string(),
    };

    client.send(hs).await;

    while let Some(result) = client.recv().await {
        match result {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
}
