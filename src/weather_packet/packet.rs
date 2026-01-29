use super::site_id::SITE_ID;

pub struct WeatherPacket {
    payload: Vec<u8>,
}

impl WeatherPacket {
    pub fn new(altitude: u8, wind_full: u8, wind_dir: f32, temp: f32, 
        humdity: f32, baro: f32) -> Self {

        let site_id = SITE_ID; //TODO read from changing file
        let mut buf = Vec::with_capacity(32);
        buf.push(site_id);
        buf.push(altitude);
        buf.push(wind_full);
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

