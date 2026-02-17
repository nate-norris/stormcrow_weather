use nmea::{parse_nmea_sentence, sentences::mda::parse_mda};
use super::models::AirmarEvent;

/// Confirm Power On Self Test (POST) response is clear.
/// If clear all values will be "0" for present sensors.
/// 
/// The airmar will provide sentences in the following structure 
///     $PAMTR,POST,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,,,,,WX
///     $PAMTR,POST,0,0,0,0,0,0,0,0,0,0,0,0,0,,0,0,,,,,WX
// pub(crate) fn interpret_post(...) -> 
//     Result<AirmarEvent, Box<dyn std::error::Error + Send + 'static>> {
// pub(crate) fn interpret_post(nmea_sentence: String)
    // -> Result<AirmarEvent, Box<dyn std::error::Error>> {
pub(crate) fn interpret_post(nmea_sentence: &str) -> anyhow::Result<AirmarEvent> {
    println!("in interpret post");
    // split comma delimited String and check for 0 at select indices
    let fields: Vec<&str> = nmea_sentence.split(',').collect();
    let zero_indices: [usize; 13] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 
        11, 12, 13, 16];

    for &i in &zero_indices {
        if fields.get(i).is_none() {
            anyhow::bail!("Malformed POST sentence, missing field at index {}", i);
            // return Err(format!("Malformed POST sentence, missing field at index {}", i).into());
        }
    }
    let all_zero: bool = zero_indices.iter().all(|&i| 
        fields.get(i) == Some(&"0"));
    
    println!("all zero? {}", all_zero);

    Ok(AirmarEvent::Post(all_zero))
}

/// Retrieve altitude provided by $PAMTC,ALT query response
/// 
/// The response is formatted as $PAMTR,ALT,<fixed altitude>,<2d mode settings>,<baro settings>
///     where the value of fixed altitude is from -999.0 to +40,000.0 meters
pub(crate) fn interpret_altitude(nmea_sentence: &str) 
    // -> Result<AirmarEvent, Box<dyn std::error::Error>> {
    -> anyhow::Result<AirmarEvent> {

    let fields: Vec<&str> = nmea_sentence.split(',').collect();
    if fields.len() != 5 {
        anyhow::bail!("Malformed altitude sentence, improper field length");
    }

    let altitude_m = fields[2].trim().parse::<f32>()?;
    
    Ok(AirmarEvent::Altitude { meters: (altitude_m) })
}

/// Retrieve weather data provided by $WIMDA sentence
/// 
/// This function uses nmea crate to parse the sentence and will return any errors
/// in creating the AirmarEvent.
pub(crate) fn interpret_wimda(nmea_sentence: &str)
    -> anyhow::Result<AirmarEvent> {

    let s = parse_nmea_sentence(nmea_sentence)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mda = parse_mda(s)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let (Some(wind_full), Some(wind_dir), Some(temp),
        Some(humidity), Some(baro),) 
        = (mda.wind_speed_ms, mda.wind_direction_magnetic, mda.air_temp_deg,
        mda.rel_humidity, mda.pressure_bar,) 
    else {
        return Err(anyhow::anyhow!("Missing required WIMDA fields"));
    };

    Ok(AirmarEvent::Wimda { 
        wind_full, 
        wind_dir, 
        temp, 
        humidity, 
        baro 
    })
}