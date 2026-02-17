use super::models::{NMEASentenceState, SOP, END_PACKET_BYTES};

pub(crate) struct NMEASentenceRetriever {
    state: NMEASentenceState,
    sentence_bytes: Vec<u8>,
}

impl NMEASentenceRetriever {

    pub fn new() -> Self {
        Self {
            state: NMEASentenceState::WaitForSOP,
            sentence_bytes: Vec::with_capacity(nmea::SENTENCE_MAX_LEN),
        }
    }

    pub fn push(&mut self, byte: u8) -> anyhow::Result<Option<String>> {

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

                // beyond max length discard the packet
                if self.sentence_bytes.len() > nmea::SENTENCE_MAX_LEN {
                    self.reset();
                    return Ok(None);
                }

                // return completed sentence
                if self.sentence_bytes.ends_with(&END_PACKET_BYTES) {

                    // remove <CR><LF>
                    let sentence_bytes = &self.sentence_bytes[..self.sentence_bytes.len() - 2];
                    let sentence = String::from_utf8(sentence_bytes.to_vec())?;

                    self.reset();
                    println!("I have a complete sentence pushed");
                    println!("{}", sentence);
                    return Ok(Some(sentence));
                }
            }
        }
        Ok(None)
    }

    pub fn reset(&mut self) {
        self.state = NMEASentenceState::WaitForSOP;
        self.sentence_bytes.clear();
    }
}