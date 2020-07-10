use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result, Responder};
use std::path::PathBuf;
use std::collections::{HashMap};
use serde_json;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn get_data(_req: HttpRequest) -> impl Responder {
    let corpus = std::fs::File::open("/home/ghpu/projets/rust/fullstack/result.json").expect("cannot open result.json");
    let corpus : common::Corpus = serde_json::from_reader(corpus).unwrap();

    web::Json(corpus)
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/data", web::get().to(get_data))
                    .route("/{filename:.*}", web::get().to(index)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
