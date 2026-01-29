use super::models::AirmarTx;
use super::trait_airmair::AirmarT;
use super::nmea_sentence::NMEASentenceRetriever;

pub struct AirmarSensorReal;

impl AirmarT for AirmarSensorReal {
    fn detect_weather_task(&self, tx: AirmarTx) -> 
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>> {

        Box::pin(async move {
            let mut port = tokio_serial::new("/dev/ttyUSB1", 4_800)
                .open_native_async()?;
            let mut buf = [0u8; 64];
            let sentence = NMEASentenceRetriever::new();

            loop {
                let n = port.read(&mut buf).await?;

                if n == 0 {
                    continue;
                }

                // process only new bytes read
                for &byte in &buf[..n] {
                    if let Some(sentence_str) = sentence.push(byte)? {
                        tx.send(sentence_str).await?;
                    }
                }
            }
        })
    }
}             