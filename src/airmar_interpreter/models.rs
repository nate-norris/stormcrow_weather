use tokio::sync::mpsc;

pub type AirmarRx = mpsc::Receiver<String>;

// interpretted nmea events
pub enum AirmarEvent {
    Post(String),
    Altitude(String),
    Weather(String),
}
// pub enum AirmarEvent {
//     Post { fields: Vec<String> },
//     Altitude { meters: f32 },
//     Weather {
//         baro_inhg: f32,
//         air_temp_c: f32,
//         humidity: f32,
//         wind_dir_mag: f32,
//         wind_speed: f32,
//     },
//     Unknown(String),
// }
// event ready for consumer
pub type AirmarEventTx = mpsc::Sender<AirmarEvent>;