use std::pin::Pin;
use tokio::time::{sleep, Duration, Sleep};
use std::future::Future;

use super::models::{AirmarEventRx, ConsumerState};
use crate::airmar::AirmarEvent;
use utils::speaker::{SpeakerTx, SpeakerNotification};
use utils::logger;


// get the transmitted nmea sentence fully parse by nmea library
// use weather_packet to form PacketT
// send over mm2t the completed packet
pub async fn airmar_consume_task<F, Fut>(mut event_rx: AirmarEventRx, speaker_tx: SpeakerTx, mut on_success: F) 
    where F: FnMut(f32, f32, f32, f32, f32, f32) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static {

    let mut state = ConsumerState::WaitingForPOST;
    let mut altitude: Option<f32> = None;
    let mut watchdog: Option<Pin<Box<Sleep>>> = None;
    let timeout = Duration::from_secs(30);

    loop{
        tokio::select! {

            Some(event) = event_rx.recv() => {
            // while let Some(event) = event_rx.recv().await {
                match (&state, &event) {
                    // failed post read
                    (ConsumerState::WaitingForPOST, AirmarEvent::Post(true)) => {
                        state = ConsumerState::WaitingForAltitude; 
                    }
                    // good post read
                    (ConsumerState::WaitingForPOST, AirmarEvent::Post(false)) => {
                        let _ = speaker_tx.send(SpeakerNotification::AirmarError).await;
                    }
                    // altitude read
                    (ConsumerState::WaitingForAltitude, AirmarEvent::Altitude { meters }) => {
                        if clear_altitude(*meters) {
                            state = ConsumerState::ReadyForWeather;
                            altitude = Some(*meters);
                            // on first init of airmar begin watchdog counter
                            watchdog = Some(Box::pin(sleep(timeout)));
                        } else {
                            logger::error("Failed to clear altitude initialization");
                            let _ = speaker_tx.send(SpeakerNotification::AirmarError).await;
                        }   
                    }
                    // weather read
                    (ConsumerState::ReadyForWeather, AirmarEvent::Wimda { wind_full, wind_dir, temp, humidity, baro }) => {
                        if clear_wimda(*wind_full, *wind_dir, *temp, *humidity, *baro) {
                            println!("clear wimda, TIMEOUT RESET");
                            // call closure
                            on_success(*wind_full, *wind_dir, *temp, *humidity, *baro, altitude.unwrap()).await;
                            // reset watchdog after every success for next timeout
                            watchdog = Some(Box::pin(sleep(timeout)));
                            // ensure timeout error is off
                            let _ = speaker_tx.send(SpeakerNotification::WeatherTimeoutError(false)).await;
                        }
                    }
                    _ => {}
                }
            }

            _ = async {
                match &mut watchdog {
                    Some(t) => {
                        t.as_mut().await;
                    },
                    None => std::future::pending::<()>().await,
                }
            } => {
                let _ = speaker_tx.send(SpeakerNotification::WeatherTimeoutError(true)).await;
                println!("TIMEOUT STARTED");
                watchdog = None;
            }
        }
    }

}

fn clear_altitude(altitude: f32) -> bool {
    altitude >= -999.9 && altitude <= 40_000.0
}

fn clear_wimda(wind_full: f32, wind_dir: f32, temp: f32, humidity: f32, baro: f32) -> bool {
    let wind_ok = wind_full >= 0.0 && wind_full <= 75.0;
    let dir_ok = wind_dir >= 0.0 && wind_dir < 360.0;
    let temp_ok = temp >= -50.0 && temp <= 140.0;
    let humidity_ok = humidity >= 0.0 && humidity <= 100.0;
    let baro_ok = baro >= 9.5 && baro <= 32.0;

    println!("Clear WIMDA: {} {} {} {} {}",
        wind_ok, dir_ok, temp_ok, humidity_ok, baro_ok);

    wind_ok && dir_ok && temp_ok && humidity_ok && baro_ok
}