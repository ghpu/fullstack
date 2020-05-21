use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpServer, Result};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}
use common::DataFromFile;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(index)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
