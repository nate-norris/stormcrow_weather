use std::pin::Pin;
use std::future::Future;

use super::models::AirmarRx;
use utils::logger;


// get the transmitted nmea sentence fully parse by nmea library
// use weather_packet to form PacketT
// send over mm2t the completed packet
pub async fn airmar_consume_task<F, Fut>(mut rx: AirmarRx, mut on_packet_complete: F)
    where F: FnMut() -> Fut,
    Fut: Future<Output = ()> + Send + 'static {
    
    loop {
        tokio::select! {

        }
    }
}