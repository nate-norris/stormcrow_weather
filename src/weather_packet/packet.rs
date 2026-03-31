//! Contains the WeatherPacket struct implementing trait PacketT
//! for uniform byte structuring of MM2T serial packets.
//! 
//! WeatherPayload defines the layout of paylod in WeatherPacket.
use utils::mm2t::{PacketT, WeatherPayload};
use super::site_id::get_site_char;

pub struct WeatherPacket {
    payload: Vec<u8>,
}

impl WeatherPacket {
    pub fn new(altitude: f32, wind_full: f32, wind_dir: f32, temp: f32, 
        humidity: f32, baro: f32) -> Self {

        assert!(get_site_char().is_ascii());
        // WeatherPayload::encode_into(self)
        let payload = WeatherPayload {
            site_id: get_site_char() as u8,
            altitude,
            wind_full,
            wind_dir,
            temp,
            humidity,
            baro,
        }
        .encode_into();
        
        Self { payload }
    }
}

impl PacketT for WeatherPacket {
    fn packet_type(&self) -> u8 {
        0x57 // character 'W' ... as in Weather
    }
    fn payload(&self) -> &[u8] {
        &self.payload
    }
}

