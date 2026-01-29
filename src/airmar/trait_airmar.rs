
use std::pin::Pin;
use std::future::Future;
use super::models::AirmarTx;

pub(crate) trait AirmarT {
    fn detect_weather_task(&self, tx: AirmarTx) -> 
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>>;
}