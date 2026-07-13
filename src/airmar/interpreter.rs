use nmea::{parse_nmea_sentence, sentences::mda::parse_mda};
use super::models::AirmarEvent;
use utils::logger;

/// Confirm Power On Self Test (POST) response is clear.
/// If clear all values will be "0" for present sensors.
/// 
/// The airmar will provide sentences in the following structure:
///     $PAMTR,POST,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,,,,,WX
///     $PAMTR,POST,0,0,0,0,0,0,0,0,0,0,0,0,0,,0,0,,,,,WX
///     WXS1500 has provided: $PAMTR,POST,0,0,0,0,0,,0,0,0,0,0,0,,,,0,,,,,WX*4D
///     <1> = Internal communication between microprocessors 
///     <2> = Format code 
///     <3> = Factory EEPROM 
///     <4> = User EEPROM 
///     <5> = Air temperature sensor 
///     <6> = Plate temperature sensor x
///     <7> = Standard relative humidity sensor 
///     <8> = Barometric pressure sensor 
///     <9> = Wind sensor 
///     <10> = Compass sensor 
///     <11> = GPS receiver 
///     <12> = Attitude sensor 
///     <13> = Rate gyro x
///     <14> = Rain sensor  x
///     <15> = Internal temperature Sensor  x
///     <16> = Battery voltage Sensor  
///     <17> = TBD Sensor  
///     <18> = TBD Sensor  
///     <19> = TBD Sensor  
///     <20> = TBD Sensor  
///     <21> = “WX” string that indicates POST results are for the family of  
///     products described in this specification.
/// 
/// 
// pub(crate) fn interpret_post(...) -> 
//     Result<AirmarEvent, Box<dyn std::error::Error + Send + 'static>> {
// pub(crate) fn interpret_post(nmea_sentence: String)
    // -> Result<AirmarEvent, Box<dyn std::error::Error>> {
pub(crate) fn interpret_post(nmea_sentence: &str) -> anyhow::Result<Option<AirmarEvent>> {

    // split comma delimited String and check for 0 at select indices
    let fields: Vec<&str> = nmea_sentence.split(',').collect();
    let zero_indices: [usize; 12] = [2, 3, 4, 5, 6, 8, 9, 10, 
        11, 12, 13, 17];

    for &i in &zero_indices {
        if fields.get(i).is_none() {
            anyhow::bail!("Malformed POST sentence, missing field at index {}", i);
        }
    }
    let all_zero: bool = zero_indices.iter().all(|&i| 
        fields.get(i) == Some(&"0"));

    if !all_zero {logger::error("Malformed POST sentence")}

    Ok(Some(AirmarEvent::Post(all_zero)))
}

/// Retrieve altitude provided by $PAMTC,ALT query response
/// 
/// The response is formatted as $PAMTR,ALT,[fixed altitude],[2d mode settings],[baro settings]
///     where the value of fixed altitude is from -999.0 to +40,000.0 meters
pub(crate) fn interpret_altitude(nmea_sentence: &str) 
    // -> Result<AirmarEvent, Box<dyn std::error::Error>> {
    -> anyhow::Result<Option<AirmarEvent>> {

    let fields: Vec<&str> = nmea_sentence.split(',').collect();
    println!("{:?}", fields);
    if fields.len() != 5 {
        anyhow::bail!("Malformed altitude sentence, improper field length");
    }

    let altitude_m = fields[2].trim().parse::<f32>()?;
    // default value set
    if altitude_m == 0.0 && fields[3] == "0" && fields[4].chars().next().unwrap() == '2' {
        return Ok(None);
    }
    
    Ok(Some(AirmarEvent::Altitude { meters: (altitude_m) }))
}

/// Retrieve weather data provided by $WIMDA sentence
/// 
/// This function uses nmea crate to parse the sentence and will return any errors
/// in creating the AirmarEvent.
pub(crate) fn interpret_wimda(nmea_sentence: &str)
    -> anyhow::Result<Option<AirmarEvent>> {

    let s = parse_nmea_sentence(nmea_sentence)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mda = parse_mda(s)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("ws:{:?}, wd:{:?}, temp:{:?}, hum:{:?}, baro:{:?}",
        mda.wind_speed_ms,
        mda.wind_direction_true,
        mda.air_temp_deg,
        mda.rel_humidity,
        mda.pressure_in_hg
    );

    let (Some(wind_full), Some(wind_dir), Some(temp),
        Some(humidity), Some(baro),) 
        = (mda.wind_speed_ms, mda.wind_direction_magnetic, mda.air_temp_deg,
        mda.rel_humidity, mda.pressure_in_hg,) 
    else {
        return Err(anyhow::anyhow!("Missing required WIMDA fields"));
    };

    Ok(Some(AirmarEvent::Wimda { 
        wind_full, 
        wind_dir, 
        temp, 
        humidity, 
        baro 
    }))
}