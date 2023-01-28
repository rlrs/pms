#[cfg_attr(target_os = "macos", path = "mac/screenshot.rs")]
#[cfg_attr(target_os = "linux", path = "linux/screenshot.rs")]
pub mod screenshot;
pub mod dhash;

pub mod api {
    tonic::include_proto!("api");
}