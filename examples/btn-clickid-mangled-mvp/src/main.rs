use std::time::Duration;

use insim::{
    Packet,
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn, BtnStyle},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (insim, _task) = insim::tcp("172.24.64.1:29999")
        .isi_iname("btn-clickid-mvp".to_string())
        .isi_flag_local(true)
        .spawn(32)
        .await?;

    let mut rx = insim.subscribe();
    tokio::spawn(async move {
        while let Ok(packet) = rx.recv().await {
            if let Packet::Btc(btc) = packet
                && btc.clickid == ClickId(5)
            {
                println!(
                    "BTC received for ClickID=5 (should stop after non-clickable update): {:?}",
                    btc
                );
            }
        }
    });

    // Clean slate for click id 5.
    insim
        .send(Bfn {
            reqi: RequestId(1),
            subt: BfnType::DelBtn,
            ucid: ConnectionId::LOCAL,
            clickid: ClickId(5),
            clickmax: 5,
            ..Default::default()
        })
        .await?;

    // Step 1: create clickable button with click id 5.
    insim
        .send(Btn {
            reqi: RequestId(5),
            ucid: ConnectionId::LOCAL,
            clickid: ClickId(5),
            l: 70,
            t: 70,
            w: 60,
            h: 10,
            text: "CLICKABLE (id=5)".to_string(),
            bstyle: BtnStyle::default().dark().yellow().clickable(),
            ..Default::default()
        })
        .await?;

    println!("Created clickable button ClickID=5. Click it now.");
    tokio::time::sleep(Duration::from_secs(4)).await;

    // This fixes it.
    // insim
    //     .send(Bfn {
    //         reqi: RequestId(1),
    //         subt: BfnType::DelBtn,
    //         ucid: ConnectionId::LOCAL,
    //         clickid: ClickId(5),
    //         clickmax: 5,
    //         ..Default::default()
    //     })
    //     .await?;

    // Step 2: update the same click id to be non-clickable.
    insim
        .send(Btn {
            reqi: RequestId(5),
            ucid: ConnectionId::LOCAL,
            clickid: ClickId(5),
            l: 70,
            t: 70,
            w: 60,
            h: 10,
            text: "NON-CLICKABLE (id=5)".to_string(),
            bstyle: BtnStyle::default().dark().yellow(),
            ..Default::default()
        })
        .await?;

    println!("Updated ClickID=5 to non-clickable. Try clicking it for 15s.");
    tokio::time::sleep(Duration::from_secs(15)).await;

    Ok(())
}
