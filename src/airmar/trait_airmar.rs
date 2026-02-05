
use std::pin::Pin;
use std::future::Future;
use super::models::{AirmarTx, SOP, END_PACKET_BYTES};
use super::nmea_sentence::NMEASentenceRetriever;

pub(crate) trait AirmarT {
    fn run(&self, tx:AirmarTx) ->
        Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>>;

    fn package_sentence(s: &str) -> Vec<u8> {
        let checksum = s.bytes().fold(0u8, |acc, 
            b| acc ^ b);
        let complete = format!("{}{}*{:02X}{}", 
            SOP, 
            s, 
            checksum,
            std::str::from_utf8(&END_PACKET_BYTES).unwrap()
        );
        complete.into_bytes()
    }

    async fn transmit_bytes(bytes: &[u8], 
        sentence_retriever: &mut NMEASentenceRetriever, 
        airmar_tx: &AirmarTx) -> anyhow::Result<()> {
            
        //iterate bytes and transmit sentence if complete
        for &byte in bytes {
            if let Some(complete_sentence) = sentence_retriever.push(byte)? {
                airmar_tx.send(complete_sentence).await?;
            }
        }
        Ok(())
    }
}