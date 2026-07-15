use insim::{
    Packet,
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn, BtnStyle},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut net = insim::tcp("172.24.64.1:29999")
        .isi_iname("btn-clickid-mvp".to_string())
        .isi_flag_local(true)
        .connect_async()
        .await?;

    // Clean slate for click id 5.
    net.write(Bfn {
        reqi: RequestId(1),
        subt: BfnType::DelBtn,
        ucid: ConnectionId::LOCAL,
        clickid: ClickId(5),
        clickmax: 5,
        ..Default::default()
    })
    .await?;

    // Step 1: create clickable button with click id 5.
    net.write(Btn {
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

    println!("Created clickable button ClickID=5. Click it now to advance to the next step.");

    loop {
        let packet = net.read().await?;
        if let Packet::Btc(btc) = packet
            && btc.clickid == ClickId(5)
        {
            println!(
                "BTC received for ClickID=5 (should stop after non-clickable update): {:?}",
                btc
            );
            break;
        }
    }

    // This fixes it.
    // net.write(Bfn {
    //     reqi: RequestId(1),
    //     subt: BfnType::DelBtn,
    //     ucid: ConnectionId::LOCAL,
    //     clickid: ClickId(5),
    //     clickmax: 5,
    //     ..Default::default()
    // })
    // .await?;

    // Step 2: update the same click id to be non-clickable.
    net.write(Btn {
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

    println!("Updated ClickID=5 to non-clickable. Try clicking it. You shouldn't be able to.");

    loop {
        let packet = net.read().await?;
        if let Packet::Btc(btc) = packet
            && btc.clickid == ClickId(5)
        {
            println!(
                "BTC received for ClickID=5 after non-clickable update: {:?}",
                btc
            );
            break;
        }
    }

    Ok(())
}
