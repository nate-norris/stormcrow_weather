//! Contains the WeatherPacket struct implementing trait PacketT
//! for uniform byte structuring of MM2T serial packets.
//! 
//! WeatherPayload defines the layout of paylod in WeatherPacket.
use utils::mm2t::PacketT;
use super::site_id::get_site_char;

#[repr(C)]
#[derive(Clone, Copy)]
struct WeatherPayload {
    site: u8,
    altitude: f32,
    wind_full: f32,
    wind_dir: f32,
    temp: f32,
    humidity: f32,
    baro: f32,
}

impl WeatherPayload {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(25);

        buf.push(self.site);
        buf.extend(self.altitude.to_le_bytes());
        buf.extend(self.wind_full.to_le_bytes());
        buf.extend(self.wind_dir.to_le_bytes());
        buf.extend(self.temp.to_le_bytes());
        buf.extend(self.humidity.to_le_bytes());
        buf.extend(self.baro.to_le_bytes());

        buf
    }
}

pub struct WeatherPacket {
    payload: Vec<u8>,
}

impl WeatherPacket {
    pub fn new(altitude: f32, wind_full: f32, wind_dir: f32, temp: f32, 
        humidity: f32, baro: f32) -> Self {

        assert!(get_site_char().is_ascii());
        let payload = WeatherPayload {
            site: get_site_char() as u8,
            altitude,
            wind_full,
            wind_dir,
            temp,
            humidity,
            baro,
        }
        .encode();
        
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

