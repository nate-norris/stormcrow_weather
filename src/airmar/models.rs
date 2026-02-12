use tokio::sync::mpsc;

pub(crate) const SOP: u8 = 0x24; // start of packet $
pub(crate) const CHECKSUM_DELIMITER: u8 = 0x2A; // "*" precedes checksum
pub(crate) const END_PACKET_BYTES: [u8; 2] = [b'\r', b'\n']; //<CR><LF>

pub(crate) enum NMEASentenceState {
    WaitForSOP, // $
    ReadSentence,
}

pub enum AirmarEvent {
    Post(bool),
    Altitude { meters: f32 },
    Wimda {
        wind_full: f32,
        wind_dir: f32,
        temp: f32,
        humidity: f32,
        baro: f32,
    },
}
// event ready for consumer
pub type AirmarEventTx = mpsc::Sender<AirmarEvent>;

pub(crate) enum ExpectedSentence {
    Post,
    Alt,
    Wimda
}
impl ExpectedSentence {
    pub(crate) fn prefix(&self) -> &'static str {
        match self {
            ExpectedSentence::Post => "$PAMTR,POST",
            ExpectedSentence::Alt => "$PAMTR,ALT",
            ExpectedSentence::Wimda => "$WIMDA",
        }
    }
}