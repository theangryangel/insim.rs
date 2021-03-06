extern crate insim;

#[tokio::main]
pub async fn main() {

    let mut client = insim::client::Client::connect(
        "insim.rs".to_string(),
        "192.168.0.250:29999".to_string()
    ).await;

    while let Some(result) = client.recv().await {
        match result {
            Err(e) => println!("{:?}", e),
            _ => ()
        }
    }

}
