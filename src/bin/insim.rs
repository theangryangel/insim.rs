extern crate insim;

#[tokio::main]
pub async fn main() {

    let mut client = insim::client::Client::connect(
        "insim.rs".to_string(),
        "isrelay.lfs.net:47474".to_string()
    ).await;

    let hlr = insim::proto::Insim::RELAY_HLR {
        reqi: 0,
        sp0: 0,
    };

    client.send(hlr).await;

    while let Some(result) = client.recv().await {
        match result {
            Err(e) => println!("{:?}", e),
            _ => ()
        }
    }

}
