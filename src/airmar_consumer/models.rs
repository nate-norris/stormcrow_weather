use tokio::sync::mpsc;

pub type AirmarRx = mpsc::Receiver<String>;