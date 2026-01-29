
// use super::trait_airmair::AirmarT;

// pub struct AirmarSensorMock;

// impl AirmarT for AirmarMock {
//     fn detect_weather_task(&self, tx: AirmarTx) -> 
//         Pin<Box<dyn Future<Output= anyhow::Result<()>> + Send>> {
//             loop {
//                 // Simulate multiple triggers
//                 for _ in 0..fastrand::u64(1..=5) {
//                     println!("trigger simulate send");
//                     tx.send(EdgeDetection::Triggered).await.unwrap();
//                     sleep(Duration::from_millis(50)).await;
//                 }
                
//                 // waiting period until next trigger occurs
//                 let wait = Duration::from_secs(fastrand::u64(10..=60));
//                 println!("Next fake trigger in: {:?} sec", wait);
//                 sleep(wait).await;
//             }
//     }
// }