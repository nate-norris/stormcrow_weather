
use std::pin::Pin;
use std::future::Future;
use super::models::AirmarTx;

pub(crate) trait AirmarT {
    fn run(&self, tx:AirmarTx) ->
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>>;
}