use rand::Rng;

use super::trait_airmair::AirmarT;
use super::nmea_sentence::NMEASentenceRetriever;
use super::models::{AirmarTx, SOP, END_PACKET_BYTES};

pub struct AirmarSensorMock;

impl AirmarT for AirmarSensorMock {

    fn run(&self, tx:AirmarTx) ->
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            let mut retriever = NMEASentenceRetriever::new();

            // mock communication with airmar for one time
            let bytes = AirmarSensorMock.package_sentence(&mock_gpgga_body());
            let body = mock_gpgga_body();
            send_sentence_body(&body, &mut retriever, &tx).await?;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            loop {
                let body = mock_wimda_body();
                send_sentence_body(&body, &mut retriever, &tx).await?;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        })
    }
}

async fn send_bytes

async fn send_sentence_body(body_string: &str,  retriever: 
    &mut NMEASentenceRetriever, tx: &AirmarTx) -> anyhow::Result<()> {

    let checksum = body_string.bytes().fold(0u8, |acc, b| acc ^ b);
    let complete = format!("{}{}*{:02X}{}", SOP, body_string, checksum, END_PACKET_BYTES);

    for byte in complete.as_bytes() {
        if let Some(complete_sentence) = retriever.push(*byte) {
            tx.send(complete_sentence.to_owned()).await?;
        }
    }

    Ok(())
}

fn mock_gpgga_body() -> String {
    let mut rng = rand::rng();

    format!("GPGGA,123456,3732.00000,N,12158.00000,E,0,2,2,{:0},M,,M,,",
        rng.gen_range(2800,3200),
    )
}

fn mock_wimda_body() -> String {
    let mut rng = rand::rng();

    format!("WIMDA,{.2},I,0.000,B,{:.1},C,,,{:.1},,0.0,C,0.0,T,{:.1},M,0.0,N,{:.1},M",
        rng.gen_range(28.00..31.00), //inHg baro inches mercury
        rng.gen_range(18.0..25.0), // air temperature Celsius
        rng.gen_range(0.0..80.0), //relative humidity
        rng.gen_range(0.0..359.9), // wind dir magnetic
        rng.gen_range(0.0..25.0), // wind speed
    )
}