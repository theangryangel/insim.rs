extern crate insim;

#[tokio::main]
pub async fn main() {
    let mut client = insim::Config::default().relay().build().await;

    // This is going to get awful to work with.
    // Is it better to have some kind of "Sink" or "Handler" thats passed to client?
    while let Some(event) = client.recv().await {
        match event {
            Ok(insim::client::Event::Connected) => {
                println!("Connected");
                let hlr = insim::packets::Insim::RelayHostListRequest(
                    insim::packets::relay::HostListRequest { reqi: 0 },
                );

                client.send(hlr);

                let hs =
                    insim::packets::Insim::RelayHostSelect(insim::packets::relay::HostSelect {
                        reqi: 0,

                        hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
                        admin: "".into(),
                        spec: "".into(),
                    });

                client.send(hs);
            }
            Ok(data) => {
                println!("{:?}", data);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}
