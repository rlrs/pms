use pms::api::pms_service_client::PmsServiceClient;
use pms::api::UploadScreenRequest;

use futures::future;
use futures_util::stream;
use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageBuffer};
use pms::screenshot::{all_screens, capture_screen};
use rgb::*;
use std::time::{Duration, SystemTime};
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PmsServiceClient::connect("http://[::1]:50001").await?;

    let screen_task = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            // wait for tick
            interval.tick().await;

            // take screenshots and send
            let screen_time = prost_types::Timestamp {
                seconds: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                nanos: 0,
            };
            let screens = all_screens();
            if screens.len() == 0 {
                eprintln!("Error: No screens found");
                break;
            }
            for screen in screens {
                let mut jpeg_data = Vec::new();
                {
                    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_data, 50);
                    let cg_image = capture_screen(screen.id).unwrap();
                    let image_data = cg_image.data();
                    let image_bgra8_data = image_data.as_bgra();
                    // convert to rgba8     TODO: better way of doing this??
                    let mut image_rgba8_data = Vec::<u8>::new();
                    for pixel in image_bgra8_data {
                        image_rgba8_data.push(pixel.r);
                        image_rgba8_data.push(pixel.g);
                        image_rgba8_data.push(pixel.b);
                        image_rgba8_data.push(pixel.a);
                    }
                    let image = DynamicImage::ImageRgba8(
                        ImageBuffer::from_vec(
                            cg_image.width() as u32,
                            cg_image.height() as u32,
                            image_rgba8_data,
                        )
                        .unwrap(),
                    );
                    encoder.encode_image(&image).unwrap();
                }
                let upload_screen_request = UploadScreenRequest {
                    time: Some(screen_time.clone()),
                    screen_id: screen.id,
                    image: jpeg_data,
                };
                let request =
                    tonic::Request::new(stream::once(async move { upload_screen_request }));
                match client.upload_screen(request).await {
                    Ok(_) => continue,
                    Err(_) => eprintln!("Error sending screen"),
                }
            } // end for screen in screens
        }
    });

    future::join_all(vec![screen_task]).await;

    Ok(())
}
