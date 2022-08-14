use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use magick_rust::MagickWand;
use mpris::{Metadata, PlayerFinder};
use qmk_oled_api::screen::OledScreen32x128;

#[derive(Debug, PartialEq, Clone)]
struct HIDSongMetadata {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub album_art_url: Option<String>,
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

impl Default for HIDSongMetadata {
    fn default() -> Self {
        Self {
            title: "No Title".to_string(),
            album: "No Album".to_string(),
            artist: "No Artist".to_string(),
            album_art_url: None,
        }
    }
}

impl From<mpris::Metadata> for HIDSongMetadata {
    fn from(metadata: mpris::Metadata) -> Self {
        let mut title = metadata.title().unwrap_or_default();
        if title.is_empty() {
            title = "No Title"
        }

        let mut album_name = metadata.album_name().unwrap_or_default();
        if album_name.is_empty() {
            album_name = "No Album"
        }

        let mut album_artists = metadata.album_artists().unwrap_or_default().join(", ");
        if album_artists.is_empty() {
            album_artists = "No Artists".to_string()
        }

        HIDSongMetadata::new(
            title.to_string(),
            album_name.to_string(),
            album_artists,
            metadata.art_url(),
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let vendor_id: u16 =
        u16::from_str_radix(env::var("VENDOR_ID").unwrap().trim_start_matches("0x"), 16).unwrap();
    let product_id: u16 =
        u16::from_str_radix(env::var("PRODUCT_ID").unwrap().trim_start_matches("0x"), 16).unwrap();
    let usage_page: u16 =
        u16::from_str_radix(env::var("USAGE_PAGE").unwrap().trim_start_matches("0x"), 16).unwrap();

    let mut screen = OledScreen32x128::from_id(vendor_id, product_id, usage_page)?;

    let client = reqwest::blocking::Client::builder().build()?;

    let image_buffer_dir = tempfile::tempdir()?;
    let image_buffer_path = image_buffer_dir.path().join("albumart.png");

    let mut tick = 0;

    let mut last_metadata: Option<HIDSongMetadata> = None;

    loop {
        let metadata = get_current_metadata()
            .unwrap_or_else(|_| Some(HIDSongMetadata::default()))
            .unwrap_or_default();

        if tick == 50 || last_metadata != Some(metadata.clone()) {
            tick = 0
        }
        last_metadata = Some(metadata.clone());

        let wand = MagickWand::new();
        let mut image_buffer = File::create(&image_buffer_path)?;

        let image_url = metadata.album_art_url;

        if let Some(image_url) = image_url {
            // Spotify haven't fixed their Linux client :(
            let image_url = image_url.replace("https://open.spotify.com/", "https://i.scdn.co/");

            let image_bytes = client.get(image_url).send()?.bytes()?;
            wand.read_image_blob(image_bytes)?;
            image_buffer.write_all(&wand.write_image_blob("png")?)?;
            screen.draw_image(&image_buffer_path, 0, 95, true);
        } else {
            screen.draw_text("?", 10, 100, 30.0, None);
        }

        let title = metadata.title + "    ";
        let title_min_index = tick % (title.len() - 4);
        let album = metadata.album + "    ";
        let album_min_index = tick % (album.len() - 4);
        let artist = metadata.artist + "    ";
        let artist_min_index = tick % (artist.len() - 4);

        screen.draw_text(
            &title[title_min_index..title_min_index + 4],
            0,
            80,
            12.0,
            None,
        );
        screen.draw_text(
            &album[album_min_index..album_min_index + 4],
            0,
            60,
            12.0,
            None,
        );
        screen.draw_text(
            &artist[artist_min_index..artist_min_index + 4],
            0,
            40,
            12.0,
            None,
        );
        tick += 1;

        screen.send()?;
        image_buffer.set_len(0)?;

        screen.clear();
        std::thread::sleep(Duration::from_millis(200));
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
