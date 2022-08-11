use std::env;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use hidapi::{HidApi};
use image_convert::{ImageResource, to_png, PNGConfig};
use mpris::{Metadata, PlayerFinder};
use qmk_oled_api::screen::OledScreen32x128;

#[derive(Debug)]
struct HIDSongMetadata {
    title: String,
    album: String,
    artist: String,
    album_art_url: Option<String>
}

impl HIDSongMetadata {
    fn new(title: String, album: String, artist: String, album_art_url: Option<&str>) -> Self {
        Self {
            title,
            album,
            artist,
            album_art_url: album_art_url.map(|s| s.to_string()),
        }
    }
}

impl From<mpris::Metadata> for HIDSongMetadata {
    fn from(metadata: mpris::Metadata) -> Self {
        HIDSongMetadata::new(
            metadata.title().unwrap_or("No Title").to_string(),
            metadata.album_name().unwrap_or("No Album").to_string(),
            metadata
                .album_artists()
                .map(|inner| inner.join(","))
                .unwrap_or_else(|| "No Artists".to_string()),
            metadata.art_url()
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let device_path =
        CString::new(env::var("DEVICE_PATH").expect("Missing required env var")).unwrap();

    let hid_api = HidApi::new().unwrap();
    let device = hid_api.open_path(&device_path).unwrap();
    let mut screen = OledScreen32x128::new();

    let client = reqwest::blocking::Client::builder().user_agent("QMK Fancy Keyboard").build()?;

    let image_buffer_dir = tempfile::tempdir()?;
    let image_buffer_path = image_buffer_dir.path().join("albumart");
    let image_buffer_converted_path = image_buffer_dir.path().join("albumart_converted.png");
    println!("{image_buffer_path:#?}");
    let mut image_buffer = File::create(&image_buffer_path)?;
    loop {

        let metadata = get_current_metadata().unwrap();
        println!("{metadata:#?}");
        let image_url = metadata.unwrap().album_art_url.unwrap().replace("https://open.spotify.com/", "https://i.scdn.co/");
        println!("{image_url}");
        let image_bytes = client.get(image_url).send()?.bytes()?;
        image_buffer.write_all(&image_bytes)?;

        let original = ImageResource::from_path(&image_buffer_path);
        let mut converted = ImageResource::from_path(&image_buffer_converted_path);
        to_png(&mut converted, &original, &PNGConfig::new())?;

        screen.clear();
        screen.draw_image(&image_buffer_converted_path, 0, 96, true);

        screen.send(&device)?;
        image_buffer.set_len(0)?;
        std::thread::sleep(Duration::from_millis(1000));
    }
}

fn get_current_metadata() -> Result<Option<HIDSongMetadata>, Box<dyn Error>> {
    let player_finder = PlayerFinder::new().map_err(|err| err.to_string())?;

    let players = player_finder.find_all().map_err(|err| err.to_string())?;

    let metadata: Option<Metadata> = players
        .iter()
        .map(|player| player.get_metadata())
        .filter_map(|metadata| metadata.ok())
        .find(|metadata| {
            if let Some(length) = metadata.length_in_microseconds() {
                length != 0
            } else {
                false
            }
        });

    if let Some(metadata) = metadata {
        Ok(Some(metadata.into()))
    } else {
        Ok(None)
    }
}
