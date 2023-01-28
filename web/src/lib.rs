pub mod search;
pub mod results;
pub mod text_input;

pub mod api {
    tonic::include_proto!("api");
}
