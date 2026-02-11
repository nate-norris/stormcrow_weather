use std::sync::Arc;
use tokio::sync::mpsc;

mod airmar;
mod airmar_consumer;

use utils::mm2t::{MM2TTransport, PacketT};
use utils::logger;
use utils::speaker::{SpeakerTx, SpeakerNotification};
use airmar::{AirmarTx, AirmarSensorReal, AirmarSensorMock};
use airmar_consumer::AirmarRx;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logger(None);
    logger::info("Weather started");

    let (airmar_tx, airmar_rx): (AirmarTx, AirmarRx)
        = mpsc::channel(32);

    // speaker for SpeakerNotifications to listen for SpeakerTx
    // NOTE initialize before any tx can transmit
    let speaker_tx: SpeakerTx = init_speaker();

    // serial radio packets
    //  NOTE: failed init here is a failed program and will 
    //  notify through SpeakerNotification
    let mm2t: Option<Arc<MM2TTransport>> = init_mm2t(&speaker_tx).await?.ok();

    if let Some(m) = mm2t {
        // initiate AirmarTx for weather detection
        spawn_airmar_detector(airmar_tx.clone(), speaker_tx.clone());

        // initiate AirmarRx to listen for weather sentences
        //      sends mm2t packet
        //      handles SpekaerNotifications
        spawn_airmar_consumer(airmar_rx, m, speaker_tx.clone());
    }
    Ok(())
}

// Initialize the speaker channels and notification task.
// Begins consuming Rx channel and returns Tx channel
fn init_speaker() -> SpeakerTx {
    // speaker channels for SpeakerNotification events
    let (speaker_tx, speaker_rx): 
        (SpeakerTx, SpeakerRx) = mpsc::channel(32);

    // await for SpeakerNotification
    // NOTE: speaker is initialized before tasks so that it is ready to receive
    tokio::spawn(speaker_consume_task(speaker_rx));

    speaker_tx
}

// Initializes MM2T radio
// On failure begins a SpeakerNotification::RadioError
async fn init_mm2t(speaker_tx: &SpeakerTx) -> Option<Arc<MM2TTransport>> {
    match MM2TTransport::start("/dev/ttyUSB0").await {
        Ok(r) => Some(Arc::new(r)),
        Err(e) => {
            logger::error("Failed mm2t init", Some(&e));
            let _ = speaker_tx.send(SpeakerNotification::RadioError).await;
            None
        }
    }
}

fn spawn_airmar_detector(tx: AirmarTx, speaker_tx: SpeakerTx) {
    let _airmar = AirmarSensorReal;
    let airmar = AirmarSensorMock;

    tokio::spawn(async move {
        // initiate transmitting altitude and weather
        if let Err(e) = airmar.run(tx.clone()).await {
            logger::error(
                "Failed airmar",
                Some(e)
            );
            let _ = speaker_tx.send(SpeakerNotification::AirmarError).await;
        }
    });
}

fn spawn_airmar_consumer(rx: AirmarRx, mm2t: Arc<MM2TTransport>, 
    speaker_tx: SpeakerTx) {
    tokio::spawn(async move {
        let speaker_tx = speaker_tx.clone();
        //TODO
    });
}