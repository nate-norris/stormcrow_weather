use tokio::sync::mpsc;

use crate::airmar_interpreter::AirmarEvent;

pub type AirmarEventRx = mpsc::Receiver<AirmarEvent>;