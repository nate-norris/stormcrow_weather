use nmea::{parse_nmea_sentence, sentences::mda::parse_mda};
use super::models::AirmarEvent;
use utils::logger;

/// Confirm Power On Self Test (POST) response is clear.
/// If clear all values will be "0" for present sensors.
/// 
/// The airmar will provide sentences in the following structure:
///     $PAMTR,POST,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,,,,,WX
///     $PAMTR,POST,0,0,0,0,0,0,0,0,0,0,0,0,0,,0,0,,,,,WX
///     WXS150 has provided: $PAMTR,POST,0,0,0,0,0,,0,0,0,0,0,0,,,,0,,,,,WX*4D
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

/// Retrieve altitude provided from notifications for PAMTC,EN,GGA
/// 
/// Sample sentence:
/// $GPGGA,201211.90,3322.1556,N,11715.8322,W,1,9,0.8,219.8,M,-34.5,M,,*65
/// 
/// GPGGA is formatted as:
///     <1> UTC of position, in the form hhmmss 
///     <2> Latitude, to the nearest .0001 minute
///     <3> N if field <2> is North Latitude 
///         S if field <2> is South Latitude 
///     <4> Longitude, to the nearest .0001 minute 
///     <5> E if field <4> is East Longitude 
///         W if field <4> is West Longitude 
///     <6> GPS quality indicator: 
///         0 = Fix not available or invalid 
///         1 = GPS SPS Mode, fix valid 
///         2 = Differential GPS, SPS Mode, fix valid 
///         3 = GPS PPS Mode, fix valid 
///         4 = Real Time Kinematic (RTK) 
///         5 = Float RTK 
///         6 = Estimated (dead reckoning) Mode 
///         7 = Manual Input Mode 
///         8 = Simulator Mode 
/// 
///         When providing data from the WX Series WeatherStation Sensor’s 
///         internal GPS, the only valid values for the GPS quality indicator 
///         are 0, 1, and 2. 
///     <7> Number of satellites in use, 0-12 
///     <8> Horizontal dilution of precision (HDOP) 
///     <9> Altitude relative to mean-sea-level (geoid), meters (to the nearest 
///         whole meter) 
///     <10> M 
///     <11> Geoidal separation, meters (to the nearest whole meter).  In the WX 
///         Series WeatherStation Sensor, this field contains the separation 
///         data, if available, otherwise, it is not provided, and appears as a 
///         null field. 
///     <12> M.  In the WX Series WeatherStation Sensor, this field contains M, 
///         if separation data is available, otherwise, it is not provided, and 
///         appears as a null field. 
///     <13> Age of Differential GPS data, seconds. This field is not provided 
///         by the WX Series WeatherStation Sensor, and appears as a null field.
///     <14> Differential reference station ID, 0000-1023.  This field is not 
///         provided by the WX Series WeatherStation Sensor, and appears as a 
///         null field
pub(crate) fn interpret_altitude(nmea_sentence: &str) 
    -> anyhow::Result<Option<AirmarEvent>> {

    let fields: Vec<&str> = nmea_sentence.split(',').collect();
    println!("{:?}", fields);
    if fields.len() != 15 {
        anyhow::bail!("Malformed altitude sentence, improper field length");
    }

    let altitude_m = fields[9].trim().parse::<f32>()?;
    
    Ok(Some(AirmarEvent::Gga { meters: (altitude_m) }))
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