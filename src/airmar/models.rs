use tokio::sync::mpsc;

pub(crate) const SOP: u8 = 0x24; // start of packet $
pub(crate) const CHECKSUM_DELIMITER: u8 = 0x2A; // "*" precedes checksum
pub(crate) const END_PACKET_BYTES: [u8; 2] = [b'\r', b'\n']; //<CR><LF>

pub(crate) enum NMEASentenceState {
    WaitForSOP, // $
    ReadSentence,
}

pub type AirmarTx = mpsc::Sender<String>;