
use std::pin::Pin;
use std::future::Future;
use super::models::{AirmarEventTx, SOP, CHECKSUM_DELIMITER, END_PACKET_BYTES};
use super::nmea_sentence::NMEASentenceRetriever;

pub trait AirmarT {

    fn run<'a>(&'a self, tx: AirmarEventTx)
        -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn package_sentence(s: &str) -> Vec<u8> {
        let checksum = s.bytes().fold(0u8, |acc, 
            b| acc ^ b);
        let mut complete = format!("{}{}{}{:02X}", 
            SOP, 
            s,
            CHECKSUM_DELIMITER,
            checksum,
        ).into_bytes();
        complete.extend_from_slice(&END_PACKET_BYTES);
        complete
    }

    fn await_retriever_sentence(bytes: &[u8], 
        sentence_retriever: &mut NMEASentenceRetriever) 
        -> anyhow::Result<Option<String>> {
        
        for &byte in bytes {
            if let Some(complete_sentence) = sentence_retriever.push(byte)? {
                return Ok(Some(complete_sentence))
            }
        }

        Ok(None)
    }
}