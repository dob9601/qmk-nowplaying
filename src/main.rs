use std::env;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use hidapi::HidApi;
use magick_rust::MagickWand;
use mpris::{Metadata, PlayerFinder};
use qmk_oled_api::screen::OledScreen32x128;

#[derive(Debug)]
struct HIDSongMetadata {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub album_art_url: Option<String>
}

struct ScrollingText {
    text: String,
    step: usize,
    window_size: usize,
}

impl ScrollingText {
    pub fn new(text: &str, window_size: usize) -> Self {
        ScrollingText {
            text: text.to_owned(),
            step: 0,
            window_size,
        }
    }

    pub fn step(&mut self) -> String {
        let output: String = self.text.chars().skip(self.step).take(self.window_size).collect();
        self.step += 1;
        if self.step > self.text.len() - 1 {
            self.step = 0;
        }
        output
    }
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
    let image_buffer_path = image_buffer_dir.path().join("albumart.png");

    loop {
        let wand = MagickWand::new();
        let mut image_buffer = File::create(&image_buffer_path)?;

        let metadata = get_current_metadata().unwrap();
        let image_url = metadata.unwrap().album_art_url.unwrap().replace("https://open.spotify.com/", "https://i.scdn.co/");
        let image_bytes = client.get(image_url).send()?.bytes()?;
        wand.read_image_blob(image_bytes)?;
        image_buffer.write_all(&wand.write_image_blob("png")?)?;

        screen.clear();
        screen.draw_image(&image_buffer_path, 0, 95, true);

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
