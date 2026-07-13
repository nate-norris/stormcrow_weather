//! read byte(s)
//! ↓
//! push into NMEASentenceRetriever
//! ↓
//! complete sentence?
//! ↓
//! check prefix (matches_expected)
//! ↓
//! interpret into AirmarEvent
//! ↓
//! tx.send(event)
use std::pin::Pin;
use std::str::MatchIndices;
use tokio_serial::{SerialStream, SerialPortBuilderExt, DataBits, Parity, 
    StopBits, FlowControl};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{timeout, Duration};

use super::models::{AirmarEventTx, ExpectedSentence};
use super::interpreter::{interpret_post, interpret_altitude, interpret_wimda,
    interpret_gga};
use super::trait_airmar::AirmarT;
use super::nmea_sentence::NMEASentenceRetriever;
use crate::logger;

pub struct AirmarSensorReal;

impl AirmarT for AirmarSensorReal {

    fn run<'a>(&'a self, tx: AirmarEventTx)
        -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {

        Box::pin(async move {
            // setup port
            let port_builder = tokio_serial::new("/dev/ttyUSB1", 4_800)
                .data_bits(DataBits::Eight)
                .parity(Parity::None)
                .stop_bits(StopBits::One)
                .flow_control(FlowControl::None)
                .timeout(std::time::Duration::from_secs(3));
            let mut port = port_builder.open_native_async()?;
            // determine proper nmea sentences from bytes
            let mut sentence_retriever = NMEASentenceRetriever::new();

            // turn off all not needed sentences
            // self.configure_sentence_transmissions(&mut port).await?;
            // check gps values
            self.verify_gga(&mut port, tx.clone(), &mut sentence_retriever).await?;
            // confirm airmar powered on correctly
            // self.power_on_self_test(&mut port, tx.clone(), &mut sentence_retriever).await?;
            // query for the altitude
            // sentence_retriever.reset();
            // self.detect_altitude(&mut port, tx.clone(), &mut sentence_retriever).await?;
            // listen for weather
            // sentence_retriever.reset();
            // self.detect_weather(&mut port, tx, &mut sentence_retriever).await?;

            Ok(())
        })
    }
}

impl AirmarSensorReal {

    async fn configure_sentence_transmissions(&self, port: &mut SerialStream)
        -> anyhow::Result<()> {
        let bytes = Self::package_sentence("PAMTC,EN,ALL,0");
        port.write_all(&bytes).await?;
        Ok(())
    }

    async fn verify_gga(&self, port: &mut SerialStream,
        tx: AirmarEventTx, retriever: &mut NMEASentenceRetriever) -> anyhow::Result<()> {
        let bytes = Self::package_sentence("PAMTC,EN,GGA,1,5");
        println!("VERIFY GGA");
        println!("{:?}", &bytes);
        port.write_all(&bytes).await?;
        // $PAMTC,EN,GSA,1
        // $PAMTC,EN,RMC,1
        let mut buf = [0u8; 64];
        timeout(Duration::from_secs(20), async {
            loop {
                let n = port.read(&mut buf).await?;
                if n == 0 {
                    continue; // no bytes read
                }
                println!("{}", n);

                match Self::process_expected_sentence(
                    &buf[..n], 
                    retriever, 
                    ExpectedSentence::Gga, 
                    interpret_gga, 
                    &tx
                ).await {
                    Ok(true) => {
                        return Ok::<(), anyhow::Error>(());
                    }
                    Ok(false) => {
                        println!("sentence was not expected")
                    }
                    Err(e) => {
                        println!("process_expected_sentence error: {:#}", e);
                        return Err(e);
                    }
                }
            }
        }).await
        .map_err(|_| anyhow::anyhow!("GGA query time out"))??;

        Ok(())
    }

    async fn power_on_self_test(&self, port: &mut SerialStream,
        tx: AirmarEventTx, retriever: &mut NMEASentenceRetriever) -> anyhow::Result<()> {
        // send power on self test command
        let bytes = Self::package_sentence("PAMTC,POST");
        port.write_all(&bytes).await?;

        // read query response for 5 seconds
        let mut buf = [0u8; 64];
        timeout(Duration::from_secs(5), async {
            loop {
                let n = port.read(&mut buf).await?;
                if n == 0 {
                    continue; // no bytes read
                }

                match Self::process_expected_sentence(
                    &buf[..n], 
                    retriever, 
                    ExpectedSentence::Post, 
                    interpret_post, 
                    &tx
                ).await {
                    Ok(true) => {
                        return Ok::<(), anyhow::Error>(());
                    }
                    Ok(false) => {
                        println!("sentence was not expected")
                    }
                    Err(e) => {
                        println!("process_expected_sentence error: {:#}", e);
                        return Err(e);
                    }
                }
            }
        }).await
        .map_err(|_| anyhow::anyhow!("POST query time out"))??;

        Ok(())
    }

    async fn detect_altitude_dprc(&self, port: &mut SerialStream, tx: AirmarEventTx, 
        retriever: &mut NMEASentenceRetriever) -> anyhow::Result<()> {

        // send query for altitude response
        let bytes = Self::package_sentence("PAMTC,ALT,Q");
        port.write_all(&bytes).await?;

        // read query response for 5 seconds
        let mut buf = [0u8; 64];
        match timeout(Duration::from_secs(10), async {
            loop {
                let n = port.read(&mut buf).await?;
                if n == 0 {
                    continue; // no bytes read
                }

                if Self::process_expected_sentence(
                    &buf[..n], 
                    retriever, 
                    ExpectedSentence::Alt, 
                    interpret_altitude, 
                    &tx
                ).await? {
                    return Ok::<(), anyhow::Error>(())
                }
            }
        }).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                eprintln!("Altitude detection error: {:#}", e);
                return Err(e);
            }
            Err(_) => {
                anyhow::bail!("Altitude query timed out");
            }
        }
        // .map_err(|_| anyhow::anyhow!("Altitude query time out"))??;

        Ok(())
    }

    async fn detect_altitude(&self, port: &mut SerialStream, tx: AirmarEventTx,
        retriever: &mut NMEASentenceRetriever) -> anyhow::Result<()> {

        let result = timeout(Duration::from_secs(60), async {
            let mut buf = [0u8; 64];

            loop {
                // Query altitude
                let bytes = Self::package_sentence("PAMTC,ALT,Q");
                port.write_all(&bytes).await?;

                // Wait for the response to this query
                loop {
                    let n = port.read(&mut buf).await?;

                    if n == 0 {
                        continue;
                    }

                    if Self::process_expected_sentence(
                        &buf[..n],
                        retriever,
                        ExpectedSentence::Alt,
                        interpret_altitude,
                        &tx,
                    )
                    .await?
                    {
                        return Ok::<(), anyhow::Error>(());
                    }

                    // process_expected_sentence returned false:
                    // - not the sentence we wanted
                    // - OR altitude interpreter returned None
                    //
                    // Send another query after a delay.
                    break;
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),

            Ok(Err(e)) => {
                eprintln!("Altitude detection failed: {:#}", e);
                Err(e)
            }

            Err(_) => {
                anyhow::bail!("Altitude query timed out waiting for valid altitude");
            }
        }
    }

    async fn detect_weather(&self, port: &mut SerialStream, tx: AirmarEventTx, 
        retriever: &mut NMEASentenceRetriever) -> anyhow::Result<()> {

        // begin weather notifications
        let bytes = Self::package_sentence("PAMTC,EN,MDA,1,25");
        port.write_all(&bytes).await?;

        let mut buf = [0u8; 64];
        loop {
            let n = port.read(&mut buf).await?;
            if n == 0 {
                continue;
            }

            if let Err(e) = Self::process_expected_sentence(
                &buf[..n], 
                retriever, 
                ExpectedSentence::Wimda, 
                interpret_wimda, 
                &tx
            ).await {
                logger::error_with("WIMDA parse failed", e);
            }
        }
    }
}




