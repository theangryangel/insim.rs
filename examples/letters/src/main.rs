use std::net::SocketAddr;

use glam::I16Vec3;
use insim::{
    Packet, Result, WithRequestId,
    core::object::{
        ObjectInfo, ObjectKind,
        painted::{Letters, PaintColour},
    },
    identifiers::PlayerId,
    insim::{Axm, PmoAction, TinyType},
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let addr: SocketAddr = "172.24.64.1:29999".parse().unwrap();

    let mut connection = insim::tcp(addr)
        .isi_flag_local(true)
        .isi_iname(Some("paint".to_string()))
        .connect_async()
        .await?;

    connection.write(TinyType::Sst.with_request_id(1)).await?;
    connection.write(TinyType::Mci.with_request_id(2)).await?;

    let mut viewed = PlayerId(0);

    loop {
        match connection.read().await? {
            Packet::Sta(sta) => {
                viewed = sta.viewplid;
            },

            Packet::Mci(mci) => {
                // Find the viewed player's position in MCI
                if let Some(comp_car) = mci.info.iter().next() {
                    if comp_car.plid != viewed {
                        continue;
                    }
                    let text = "HELLO WORLD";
                    let letters =
                        Letters::from_str(text, PaintColour::White, comp_car.direction).unwrap();
                    let objects: Vec<ObjectKind> = letters
                        .into_iter()
                        .map(|l| ObjectKind::PaintLetters(l))
                        .collect();

                    // FIXME: pointless unit versions. fml. make insim.rs handle this.
                    let x = (comp_car.xyz.x / 65536) as i16 * 16 + 10;
                    let y = (comp_car.xyz.y / 65536) as i16 * 16;
                    let z = (comp_car.xyz.z / 65536) as i16 * 16;

                    // Use player's position
                    let center = I16Vec3 { x, y, z };
                    let spacing = 20;
                    let objects_info = ObjectInfo::spaced_from_center(objects, center, spacing);

                    let axm = Axm {
                        pmoaction: PmoAction::AddObjects,
                        info: objects_info,
                        ..Default::default()
                    };

                    println!("painting..{:?}", axm);

                    connection.write(axm).await.expect("should send");
                }
            },

            _ => {
                println!("woot!");
            },
        }
    }
}
