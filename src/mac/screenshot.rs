use core_graphics::display::CGDisplay;
use core_graphics::image::CGImage;
use display_info::DisplayInfo;

pub fn all_screens() -> Vec<DisplayInfo> {
    let screens = DisplayInfo::all().unwrap();
    screens
}

pub fn capture_screen(screen_id: u32) -> Option<CGImage> {
    let cg_display = CGDisplay::new(screen_id);
    let cg_image = cg_display.image()?;

    Some(cg_image)
}
