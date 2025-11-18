#![allow(unused_imports)]
mod control_protocols;
use control_protocols::screen_control::{SetInterval, Ticks};
use serde::{Serialize, Deserialize};
use std::{time::Duration, fs};
use std::env::current_dir;
use tokio_stream::StreamExt;
use zbus::{Connection, Result, proxy, zvariant::{OwnedObjectPath, Type}, Proxy};
use image::{Frame, RgbaImage};
use percent_encoding::percent_decode_str;
use zbus::{zvariant::{as_value::{self}}};
use url::Url;
use tokio;
use console_subscriber;
use tokio::time::{sleep, Instant};
use chrono::Local;


/// https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html
/// https://github.com/flatpak/xdg-desktop-portal/blob/main/data/org.freedesktop.impl.portal.Screenshot.xml
// create screenshot
#[proxy(
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop",
    interface = "org.freedesktop.portal.Screenshot"
)]
trait Screenshot {
    fn screenshot(&self, parent_window: String, options: &OptionsScreen) -> Result<OwnedObjectPath>; //HashMap<&str, Value<'_>>
}
// returns uri with screen from desktop portal
#[derive(Debug, Type, Serialize, Deserialize)]
#[zvariant(signature = "dict")]
struct ScreenshotResponse {
    #[serde(with = "as_value")]
    uri: String,
}
// Request for desktop portal
#[proxy(
    default_service = "org.freedesktop.portal.Desktop",
    interface = "org.freedesktop.portal.Request",
)]
trait Request {
    fn close(&self) -> Result<()>;
    #[zbus(signal)]
    fn response(&self, response: u32, results: ScreenshotResponse) -> Result<()>; //
}
#[derive(Debug, Type, Serialize, Deserialize)]
#[zvariant(signature = "dict")]
struct OptionsScreen {
    #[serde(with = "as_value")]
    handle_token: String,
    #[serde(with = "as_value")]
    modal: bool,
    #[serde(with = "as_value")]
    interactive: bool,
}
// option for screenshot
impl OptionsScreen {
    fn init() -> Self {
        let handle_token = rand::random::<u32>().to_string();
        Self {
            handle_token,
            modal: false,
            interactive: false,
        }
    }
}

fn uri_to_buf_img(uri: &str) -> Result<RgbaImage> {
    let url_parse = Url::parse(uri).unwrap();
    let decode_url = percent_decode_str(url_parse.path())
        .decode_utf8_lossy()
        .to_string();
    let dyn_image = image::open(&decode_url)
        .expect("error when open image path")
        .to_rgba8();

    let _ = fs::remove_file(decode_url);
    Ok(dyn_image)
}

fn uri_to_frame(uri:&str) -> Result<Frame> {
    let url_parse = Url::parse(uri).unwrap();
    let decode_url = percent_decode_str(url_parse.path())
        .decode_utf8_lossy()
        .to_string();
    let dyn_image = image::open(&decode_url)
        .expect("error when open image path")
        .to_rgba8();
    let frame = Frame::new(dyn_image);
    let _ = fs::remove_file(decode_url);
    Ok(frame)
}

struct Screen {
    interval: Duration,
    duration: Duration,
    time_begin: Instant,
}

impl Screen {
    fn init() -> Self {
        // set intervals
        let interval = SetInterval::Ticks(Ticks::Minutes);
        let time_begin = Instant::now();
        let interval = interval
            .ticks_minutes(1)
            .expect("error duration settings");
        // set durations
        let duration = SetInterval::Ticks(Ticks::Hours);
        let duration = duration
            .ticks_hours(1.0)
            .expect("error duration settings");
        Self {
            interval,
            duration,
            time_begin,
        }
    }
}

fn time_now() -> String {
    let time = chrono::Utc::now().with_timezone(&Local);
    let mut time = time.format("%Y-%m-%d %H:%M:%S").to_string();
    time.push('\n');
    time
}
// some problem with reducing  memory used with desktop portal kde....
async fn wayland_screenshot() -> Result<()> {
    let folder_name = "screens";

    if !current_dir()?.join(folder_name).exists() {
        fs::create_dir(folder_name).expect("ds");
    }
    let Screen {interval, duration, time_begin} = Screen::init();
    let options_str = OptionsScreen::init();
    let handle_token = &options_str.handle_token;
    let connection = Connection::session().await?;
    let unique_name = &connection
        .unique_name()
        .expect("should be unique name")
        .trim_start_matches(":")
        .replace(".", "_");
    let path = format!("/org/freedesktop/portal/desktop/request/{unique_name}/{handle_token}");
    let request = RequestProxy::new(&connection, path).await?;

    let proxy_screen = ScreenshotProxy::new(&connection).await?;

    // interval process
    tokio::spawn(async move {
        loop {
            sleep(interval).await;
            let _res = proxy_screen.screenshot("".to_string(), &options_str).await.unwrap();
        }
    });
    tokio::spawn( async move {
        let mut req = request.receive_response().await.expect("error response_stream");
        while let Some(msg) = req.next().await {
            let args = msg.args().expect("error response_arg");
            match args.response {
                0 => {
                    let uri = args.results.uri;
                    let image = uri_to_buf_img(uri.as_str()).expect("uri reading problem");
                    println!("11");
                    let dir = current_dir().unwrap()
                        .join(folder_name)
                        .join(time_now())
                        .with_extension("png");
                    image.save(dir).unwrap();
                }
                1 => {println!("The user cancelled")},
                _ => {eprintln!("Error: Could not complete task");}
            }
        }
    });

    loop {
        if time_begin.elapsed() > duration {
            connection.close().await?;
            break;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    //console_subscriber::init();
    wayland_screenshot().await.unwrap();
}

