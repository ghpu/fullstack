use actix_web::{web,App,HttpServer,HttpRequest, Result};
use std::path::PathBuf;
use actix_files::NamedFile;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)

}
use common::Test;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| 
        App::new().route("/{filename:.*}", web::get().to(index)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
