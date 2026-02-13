use super::models::{AirmarEventRx, ConsumerState};
use crate::airmar::AirmarEvent;
use utils::speaker::{SpeakerTx, SpeakerNotification};


// get the transmitted nmea sentence fully parse by nmea library
// use weather_packet to form PacketT
// send over mm2t the completed packet
pub async fn airmar_consume_task(mut event_rx: AirmarEventRx, speaker_tx: SpeakerTx) {

    let mut state = ConsumerState::WaitingForPOST;
    let mut altitude: Option<f32> = None;
    while let Some(event) = event_rx.recv().await {
        //TODO result to error state in certain situations?

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
                } else {
                    let _ = speaker_tx.send(SpeakerNotification::AirmarError).await;
                }   
            }
            //
            (ConsumerState::ReadyForWeather, AirmarEvent::Wimda { wind_full, wind_dir, temp, humidity, baro }) => {
                if clear_wimda(*wind_full, *wind_dir, *temp, *humidity, *baro) {

                }
            }
            _ => {}
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
    let baro_ok = baro >= 850.0 && baro <= 1100.0;

    wind_ok && dir_ok && temp_ok && humidity_ok && baro_ok
}