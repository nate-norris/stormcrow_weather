use super::models::{NMEASentenceState, SOP, END_PACKET_BYTES};

use nmea::constant::SENTENCE_MAX_LEN as MAX_LEN; // 102 characters

pub(crate) struct NMEASentenceRetriever {
    state: NMEASentenceState,
    sentence_bytes: Vec<u8>,
}

impl NMEASentenceRetriever {

    pub fn new() -> Self {
        Self {
            state: NMEASentenceState::WaitForSOP,
            sentence_bytes: Vec::with_capacity(MAX_LEN),
        }
    }

    pub fn push(&mut self, byte: u8) -> anyhow::Result<Option<&str>> {

        match self.state {
            // check bytes until at start of packet
            NMEASentenceState::WaitForSOP => {
                if byte == SOP {
                    self.sentence_bytes.clear();
                    self.state = NMEASentenceState::ReadSentence;
                }
            }

            // read the entire sentence
            NMEASentenceState::ReadSentence => {
                self.sentence_bytes.push(byte);

                // return completed sentence
                if self.sentence_bytes.ends_with(&END_PACKET_BYTES) {
                    // remove <CR><LF>
                    let sentence = std::str::from_utf8(&self.sentence_bytes[..self.sentence_bytes.len()-2])
                        .ok();

                    self.reset();
                    return(Ok(sentence));
                }
            }
        }
        Ok(None)
    }

    fn reset(&mut self) {
        self.state = NMEASentenceState::WaitForSOP;
        self.sentence_bytes.clear();
    }
}