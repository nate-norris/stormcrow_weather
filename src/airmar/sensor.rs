use tokio::serial::SerialStream;
use super::models::AirmarTx;
use super::trait_airmair::AirmarT;
use super::nmea_sentence::NMEASentenceRetriever;

pub struct AirmarSensorReal;

/*
open port
send disable-all command except WIMDA
send GGA query (one time read)
wait for GGA

stream WIMDA forever
 */

impl AirmarT for AirmarSensorReal {

    fn run(&self, tx:AirmarTx) ->
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>> {
        Box::pin(async move {
            let mut port = tokio_serial::new("/dev/ttyUSB1", 4_800)
                .open_native_async()?;

            // turn off all not needed sentences
            //      Keep WIMDA sentences active.
            //      GPGGA will be read as a query only 
            configure_sentence_transmissions(&mut port).await?;

            // query for the altitude
            detect_altitude(&mut port, tx.clone()).await?;
            
            // listen for weather
            detect_weather(&mut port, tx).await?;
            Ok(())
        })
    }
}

async fn configure_sentence_transmissions(port: &mut SerialStream) 
    -> anyhow::Result<()> {
    Ok(())
}

async fn detect_altitude(port: &mut SerialStream, tx: AirMarTx) ->
    anyhow::Result<()> {
    //$GPQ,GGA*CS\r\n
    Ok(())
}

async fn detect_weather(port: &mut SerialStream, tx: AirmarTx) -> 
    anyhow::Result<()> {

    //TODO enable weather transmissions
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
}