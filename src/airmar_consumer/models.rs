use tokio::sync::mpsc;

use crate::airmar::AirmarEvent;

pub type AirmarEventRx = mpsc::Receiver<AirmarEvent>;

pub(crate) enum ConsumerState {
    WaitingForPOST,
    WaitingForAltitude,
    ReadyForWeather,
}