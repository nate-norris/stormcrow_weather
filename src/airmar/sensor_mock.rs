//! provide bytes
//! ↓
//! push into retriever (optional but good for realism)
//! ↓
//! complete sentence
//! ↓
//! matches_expected
//! ↓
//! interpret
//! ↓
//! tx.send(event)
//! 
use std::future::Future;
use std::pin::Pin;
use rand::Rng;

use super::trait_airmar::AirmarT;
use super::nmea_sentence::NMEASentenceRetriever;
use super::models::{AirmarEventTx, ExpectedSentence};
use super::interpreter::{interpret_post, interpret_altitude, interpret_wimda};
use crate::logger;

pub struct AirmarSensorMock;

impl AirmarT for AirmarSensorMock {

    fn run<'a>(&'a self, tx: AirmarEventTx)
        -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut retriever = NMEASentenceRetriever::new();

            // send fake post response once
            let bytes = <Self as AirmarT>::package_sentence(&mock_post_body()); // fake bytes
            if !Self::process_expected_sentence(
                &bytes, 
                &mut retriever, 
                ExpectedSentence::Post, 
                interpret_post, 
                &tx
            ).await? {
                println!("failed to process mock post");
            }
            retriever.reset();
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            // send fake altitude transmission once
            let bytes = <Self as AirmarT>::package_sentence(&mock_gpgga_body());
            Self::process_expected_sentence(
                &bytes, 
                &mut retriever, 
                ExpectedSentence::Alt, 
                interpret_altitude, 
                &tx
            ).await?;
            retriever.reset();
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            // send fake weather tranmission every three seconds
            loop {
                // get fake bytes
                let bytes = <Self as AirmarT>::package_sentence(&mock_wimda_body());
                if let Err(e) = Self::process_expected_sentence(
                    &bytes, 
                    &mut retriever, 
                    ExpectedSentence::Wimda, 
                    interpret_wimda, 
                    &tx
                ).await {
                    logger::error("WIMDA parse failed", Some(e));
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        })
    }
}

/// Provides a fake POST response that would occur during airmar transmission 
/// of $PAMTC,POST.
/// 
/// Will randomly provide a failed post when a 1 is present in the String
fn mock_post_body() -> String {
    let mut rng = rand::rng();
    // TODO have a break in POST
    format!("PAMTR,POST,0,0,0,0,{},0,0,0,0,0,0,0,0,0,0,0,,,,,WX",
        rng.random_range(0..=1), // air temp sensor mock failure
    )
}

fn mock_gpgga_body() -> String {
    // TODO have a break in altitude
    let mut rng = rand::rng();

    format!("GPGGA,123456,3732.00000,N,12158.00000,E,0,2,2,{:0},M,,M,,",
        rng.random_range(2800..3200), // random altitude
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