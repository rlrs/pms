extern crate tantivy;
#[macro_use]
extern crate text_io;
use chrono::Datelike;
use indicatif::ProgressBar;
use std::collections::HashSet;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tantivy::collector::TopDocs;
use tantivy::schema::*;
use tantivy::{Document, Index, IndexWriter};
use tokio::sync::RwLock;
use tokio::time;
use tonic::{transport::Server, Request, Response, Status, Streaming};
use walkdir::WalkDir;

use pms::api;
use pms::api::pms_service_server::{PmsService, PmsServiceServer};
use pms::api::{
    search_response::Screen as SearchResponseScreen, Ack, SearchRequest, SearchResponse,
    UploadScreenRequest,
};
use pms::dhash::{get_dhash, IMG_SIZE};

use leptess::LepTess;

pub struct ImplPMSService {
    schema: Schema,
    index: Index,
    writer_arc: Arc<RwLock<IndexWriter>>,
    hashes: RwLock<HashSet<[bool; IMG_SIZE]>>,
}

impl ImplPMSService {
    fn new(schema: Schema, index: Index, writer_arc: Arc<RwLock<IndexWriter>>) -> Self {
        ImplPMSService {
            schema,
            index,
            writer_arc,
            hashes: RwLock::new(HashSet::new()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // if "screenshots/" exist, but "index/" doesn't, rebuild the index
    if std::path::Path::new(&"screenshots/").exists() && !std::path::Path::new("index/").exists() {
        println!("Rebuilding index");
        std::fs::create_dir("index/")?;
        let (schema, index) = make_schema();
        rebuild_index(&index, &schema).await;
    }
    let (schema, index) = make_schema();

    let addr = "[::1]:50001".parse()?;
    let writer: Arc<RwLock<IndexWriter>> = Arc::new(RwLock::new(index.writer(50_000_000).unwrap()));
    let service = ImplPMSService::new(schema, index, writer.clone());

    let server = Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(PmsServiceServer::new(service)))
        .serve(addr);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            print!("Committing index... ");
            writer.clone().write().await.commit().unwrap();
            println!("done.");
        }
    });

    // join all tasks
    server.await?;

    Ok(())
}

fn datetime_to_screen_path(datetime: chrono::NaiveDateTime, screen_id: u32) -> (String, String) {
    let path = format!(
        "screenshots/{}/{}/{}/",
        datetime.year(),
        datetime.month(),
        datetime.day()
    );
    let fname = format!("{}-{}.jpg", datetime.format("%H%M%S"), screen_id);
    (path, fname)
}

fn get_all_datetime_screens(datetime: chrono::NaiveDateTime) -> Vec<String> {
    // glob all screens matching datetime for all possible screen_ids
    let (path, fname) = datetime_to_screen_path(datetime, 0);
    let time = fname.split('-').next().unwrap();
    glob::glob(&format!("{}/{}-*.jpg", path, time))
        .unwrap()
        .map(|x| x.unwrap().to_str().unwrap().to_string())
        .collect()
}

#[tonic::async_trait]
impl PmsService for ImplPMSService {
    async fn upload_screen(
        &self,
        request: Request<Streaming<UploadScreenRequest>>,
    ) -> Result<Response<Ack>, Status> {
        let result: Result<(), Status> = {
            let mut stream = request.into_inner();
            let req = stream.message().await?.unwrap();
            let time = req.time.unwrap();
            let datetime =
                chrono::NaiveDateTime::from_timestamp_opt(time.seconds, time.nanos as u32).unwrap();

            // Hash the image and check if it's already in the index
            let dyn_image = image::load_from_memory(&req.image).unwrap();
            let hash = get_dhash(&dyn_image);
            {
                let hashes = self.hashes.read().await;
                if hashes.contains(&hash) {
                    return Ok(Response::new(Ack { success: true }));
                }
            }

            // OCR the image
            let text = ocr_image_mem(&req.image);

            // Save the image
            let (path, fname) = datetime_to_screen_path(datetime, req.screen_id);
            create_dir_all(&path)?; // Not handled on purpose
            let mut file = File::create(path + &fname).unwrap();
            file.write_all(&req.image).unwrap();

            // Index the image
            let mut doc = Document::default();
            doc.add_text(self.schema.get_field("text").unwrap(), &text);
            doc.add_date(
                self.schema.get_field("date").unwrap(),
                tantivy::DateTime::from_timestamp_secs(datetime.timestamp()),
            );
            doc.add_u64(
                self.schema.get_field("screen_id").unwrap(),
                req.screen_id as u64,
            );
            let index_writer = self.writer_arc.read().await;
            index_writer.add_document(doc).unwrap();

            {
                // Add the hash to the set
                let mut hashes = self.hashes.write().await;
                hashes.insert(hash);
            }

            Ok(())
        };
        let reply = match result {
            Ok(_) => Ack { success: true },
            Err(_) => Ack { success: false },
        };

        Ok(Response::new(reply))
    }

    async fn search_screens(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let result: Result<SearchResponse, Status> = {
            let req = request.into_inner();
            println!("Searching for {}", req.query);
            let query_parser = tantivy::query::QueryParser::for_index(
                &self.index,
                vec![self.schema.get_field("text").unwrap()],
            );
            let reader = self
                .index
                .reader_builder()
                .reload_policy(tantivy::ReloadPolicy::OnCommit)
                .try_into()
                .unwrap();
            let searcher = reader.searcher();
            let query = query_parser.parse_query(&req.query).unwrap();
            let top_docs = searcher.search(&query, &TopDocs::with_limit(200)).unwrap();
            let mut screens: Vec<SearchResponseScreen> = vec![];
            println!("Found {} results", top_docs.len());
            for (_score, doc_address) in top_docs {
                let retrieved_doc = searcher.doc(doc_address).unwrap();

                let text = retrieved_doc
                    .get_first(self.schema.get_field("text").unwrap())
                    .unwrap()
                    .as_text()
                    .unwrap();
                let date = retrieved_doc
                    .get_first(self.schema.get_field("date").unwrap())
                    .unwrap()
                    .as_date()
                    .unwrap();
                //let screen_id = retrieved_doc.get_first(self.schema.get_field("screen_id").unwrap()).unwrap_or(&tantivy::schema::Value::U64(0)).as_u64().unwrap();
                let screen_id = retrieved_doc
                    .get_first(self.schema.get_field("screen_id").unwrap())
                    .unwrap()
                    .as_u64()
                    .unwrap();
                let (image_path, image_fname) = datetime_to_screen_path(
                    chrono::NaiveDateTime::from_timestamp_opt(date.into_timestamp_secs(), 0)
                        .unwrap(),
                    screen_id as u32,
                );
                let image_full_path = image_path + &image_fname;
                screens.push(SearchResponseScreen {
                    screen_id: screen_id as u32,
                    image: std::fs::read(image_full_path).unwrap(),
                    text: text.to_string(),
                    time: Some(prost_types::Timestamp {
                        seconds: date.into_timestamp_secs(),
                        nanos: 0,
                    }),
                });
            }
            Ok(api::SearchResponse { screens })
        };
        Ok(Response::new(result.unwrap()))
    }
}

fn ocr_image_mem(image: &[u8]) -> String {
    let mut tess = LepTess::new(None, "eng").unwrap();
    tess.set_image_from_mem(&image).unwrap();
    tess.get_utf8_text().unwrap()
}

fn ocr_image_path(path: &str) -> String {
    let mut tess = LepTess::new(None, "eng").unwrap();
    tess.set_image(path).unwrap();
    tess.get_utf8_text().unwrap()
}

fn index_image(
    schema: &Schema,
    writer: tokio::sync::RwLockReadGuard<'_, tantivy::IndexWriter>,
    path: &str,
) {
    let mut doc = Document::default();
    let text = ocr_image_path(path);
    doc.add_text(schema.get_field("text").unwrap(), &text);
    let year: u32;
    let month: u32;
    let day: u32;
    let time: String;
    let screen_id: u64;
    scan!(path.bytes() => "screenshots/{}/{}/{}/{}-{}.jpg", year, month, day, time, screen_id);
    let time = chrono::NaiveTime::parse_from_str(&time, "%H%M%S").unwrap();
    let datetime = chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
        .unwrap()
        .and_time(time);
    doc.add_date(
        schema.get_field("date").unwrap(),
        tantivy::DateTime::from_timestamp_secs(datetime.timestamp()),
    );
    doc.add_u64(schema.get_field("screen_id").unwrap(), screen_id);
    writer.add_document(doc).unwrap();
}

async fn rebuild_index(index: &Index, schema: &Schema) {
    let writer_arc: Arc<RwLock<IndexWriter>> =
        Arc::new(RwLock::new(index.writer(50_000_000).unwrap()));
    let schema_arc = Arc::new(schema.clone());
    let mut handles = vec![];
    let pb = Arc::new(RwLock::new(ProgressBar::new(0)));
    for entry in WalkDir::new("screenshots") {
        let my_schema = Arc::clone(&schema_arc);
        let my_writer = Arc::clone(&writer_arc);
        let my_pb = Arc::clone(&pb);
        let job = tokio::spawn(async move {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                index_image(
                    &*my_schema,
                    my_writer.clone().read().await,
                    entry.path().to_str().unwrap(),
                );
            }
            my_pb.read().await.inc(1);
        });
        handles.push(job);
    }
    let count = handles.len();
    pb.read().await.inc_length(count as u64);
    futures_util::future::join_all(handles).await;
    pb.read().await.finish();
    writer_arc.clone().write().await.commit().unwrap();
    println!("Done: Indexed {} images", count);
}

fn make_schema() -> (Schema, Index) {
    const INDEX_PATH: &str = "index/";
    if !std::path::Path::new(INDEX_PATH).exists() {
        std::fs::create_dir(INDEX_PATH).unwrap();
    }
    let mut schema_builder = SchemaBuilder::default();
    let _date = schema_builder.add_date_field("date", STORED);
    let _text = schema_builder.add_text_field("text", TEXT | STORED);
    let _screen_id = schema_builder.add_u64_field("screen_id", STORED);
    let schema = schema_builder.build();

    // Create or open the tantivy index
    let dir = tantivy::directory::MmapDirectory::open(INDEX_PATH).unwrap();
    let index = Index::open_or_create(dir, schema.clone()).unwrap();

    (schema, index)
}
