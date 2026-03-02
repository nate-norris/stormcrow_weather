use utils::mm2t::PacketT;
use super::site_id::get_site_uuid;

pub struct WeatherPacket {
    payload: Vec<u8>,
}

impl WeatherPacket {
    pub fn new(altitude: f32, wind_full: f32, wind_dir: f32, temp: f32, 
        humdity: f32, baro: f32) -> Self {

        let mut buf = Vec::with_capacity(25);
        buf.push(*get_site_uuid());
        buf.extend_from_slice(&altitude.to_le_bytes());
        buf.extend_from_slice(&wind_full.to_le_bytes());
        buf.extend_from_slice(&wind_dir.to_le_bytes());
        buf.extend_from_slice(&temp.to_le_bytes());
        buf.extend_from_slice(&humdity.to_le_bytes());
        buf.extend_from_slice(&baro.to_le_bytes());
        
        Self { payload: buf }
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

