fn main() -> Result<(), Box<dyn std::error::Error>> {
  let proto_files = ["proto/main.proto"];
  tonic_build::configure()
    //.compile_well_known_types()
    .compile(&proto_files, &["."])
    .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
  
  println!("cargo:rerun-if-changed={:?}", proto_files);
  Ok(())
}