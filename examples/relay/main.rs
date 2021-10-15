extern crate insim;

#[tokio::main]
pub async fn main() {
    let mut client = insim::Config::default().relay().build().await;

    let hlr = insim::packets::Insim::RelayHostListRequest(insim::packets::relay::HostListRequest {
        reqi: 0,
    });

    client.send(hlr);

    let hs = insim::packets::Insim::RelayHostSelect(insim::packets::relay::HostSelect {
        reqi: 0,

        hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
        admin: "".into(),
        spec: "".into(),
    });

    client.send(hs);

    while let Some(event) = client.recv().await {
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
