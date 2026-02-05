use std::future::Future;
use std::pin::Pin;
use rand::Rng;

use super::trait_airmar::AirmarT;
use super::nmea_sentence::NMEASentenceRetriever;
use super::models::AirmarTx;

pub struct AirmarSensorMock;

impl AirmarT for AirmarSensorMock {

    fn run(&self, tx: AirmarTx) ->
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            let mut retriever = NMEASentenceRetriever::new();

            // send fake altitude transmission once
            let bytes = <Self as AirmarT>::package_sentence(&mock_gpgga_body());
            <Self as AirmarT>::transmit_bytes(&bytes, &mut retriever, &tx).await?;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            loop {
                // send fake weather tranmission every three seconds
                let bytes = <Self as AirmarT>::package_sentence(&mock_wimda_body());
                <Self as AirmarT>::transmit_bytes(&bytes, &mut retriever, &tx).await?;
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        })
    }
}

fn mock_gpgga_body() -> String {
    let mut rng = rand::rng();

    format!("GPGGA,123456,3732.00000,N,12158.00000,E,0,2,2,{:0},M,,M,,",
        rng.random_range(2800..3200),
    )
}

fn mock_wimda_body() -> String {
    let mut rng = rand::rng();

    format!("WIMDA,{:.2},I,0.000,B,{:.1},C,,,{:.1},,0.0,C,0.0,T,{:.1},M,0.0,N,{:.1},M",
        rng.random_range(28.00..31.00), //inHg baro inches mercury
        rng.random_range(18.0..25.0), // air temperature Celsius
        rng.random_range(0.0..80.0), //relative humidity
        rng.random_range(0.0..359.9), // wind dir magnetic
        rng.random_range(0.0..25.0), // wind speed
    )
}