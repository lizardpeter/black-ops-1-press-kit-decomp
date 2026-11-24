//! Original application behavior, independently translated from faithful C++ and AVM1 evidence.
//! No GPU, filesystem, audio or platform dependency.
pub mod gallery;
pub mod assets;
pub mod layout;
pub mod timeline;
pub use gallery::Gallery;
pub use layout::{Hit, hit_menu, hit_launcher, hit_media, Stage};
pub use timeline::{App, Effect, Scene, Background, Controls};

pub const INTRO: &str = "payload/intro/IntroSeq.mp4";
pub const WEBSITE: &str = "http://www.callofduty.com";
pub const SCREEN_XML: &str = "payload/screens/screens.xml";
pub const WALLPAPERS: [&str; 8] = [
    "payload/wallpaper/1024x768/COD7-Wallpaper-1024x768.jpg",
    "payload/wallpaper/1280x800/COD7-Wallpaper-1280x800.jpg",
    "payload/wallpaper/1280x1024/COD7-Wallpaper-1280x1024.jpg",
    "payload/wallpaper/1440x900/COD7-Wallpaper-1440x900.jpg",
    "payload/wallpaper/1680x1050/COD7-Wallpaper-1680x1050.jpg",
    "payload/wallpaper/1600x1200/COD7-Wallpaper-1600x1200.jpg",
    "payload/wallpaper/1920x1200/COD7-Wallpaper-1920x1200.jpg",
    "payload/wallpaper/2560x1440/COD7-Wallpaper-2560x1440.jpg",
];
pub fn trailer(id: u8) -> Option<&'static str> {
    Some(match id {
        1 => "payload/movies/1.BlackOpsMPTeaser.mp4",
        2 => "payload/movies/2.PrestigeEditionVideo.mp4",
        4 => "payload/movies/4.UncutRevealTrailer.mp4",
        5 => "payload/movies/5.DebutTeaseTrailer.mp4",
        6 => "payload/movies/6.Wagermatch.mp4",
        _ => return None,
    })
}
pub fn flash_time(seconds: f64) -> String {
    if !seconds.is_finite() { return "NaN:NaN".into(); }
    let rounded = (seconds + 0.5).floor();
    format!("{:02}:{:02}", (rounded / 60.).floor() as i64, (rounded % 60.).floor() as i64)
}
