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
use tokio_serial::{SerialStream, SerialPortBuilderExt, DataBits, Parity, 
    StopBits, FlowControl};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{timeout, Duration};

use super::models::{AirmarEventTx, ExpectedSentence};
use super::interpreter::{interpret_post, interpret_altitude, interpret_wimda};
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
            self.configure_sentence_transmissions(&mut port).await?;
            // confirm airmar powered on correctly
            self.power_on_self_test(&mut port, tx.clone(), &mut sentence_retriever).await?;
            // query for the altitude
            sentence_retriever.reset();
            self.detect_altitude(&mut port, tx.clone(), &mut sentence_retriever).await?;
            // listen for weather
            sentence_retriever.reset();
            self.detect_weather(&mut port, tx, &mut sentence_retriever).await?;

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

                if Self::process_expected_sentence(
                    &buf[..n], 
                    retriever, 
                    ExpectedSentence::Post, 
                    interpret_post, 
                    &tx
                )? {
                    return Ok::<(), anyhow::Error>(())
                }
            }
        }).await
        .map_err(|_| anyhow::anyhow!("POST query time out"))??;

        Ok(())
    }

    async fn detect_altitude(&self, port: &mut SerialStream, tx: AirmarEventTx, 
        retriever: &mut NMEASentenceRetriever) -> anyhow::Result<()> {

        // send query for altitude response
        let bytes = Self::package_sentence("PAMTC,ALT,Q");
        port.write_all(&bytes).await?;

        // read query response for 5 seconds
        let mut buf = [0u8; 64];
        timeout(Duration::from_secs(5), async {
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
                )? {
                    return Ok::<(), anyhow::Error>(())
                }
            }
        }).await
        .map_err(|_| anyhow::anyhow!("Altitude query time out"))??;

        Ok(())
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
            ) {
                logger::error("WIMDA parse failed", Some(e));
            }
        }
    }
}




