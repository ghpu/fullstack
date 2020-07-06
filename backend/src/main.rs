use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result, Responder};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn get_data(_req: HttpRequest) -> impl Responder {
    let case = common::Case {text:"Joue du Joan Baez".to_string(), gold:vec![common::Annotation{intent:"Music_play".to_string(), values:vec![]}], a:vec![], b:vec![]};
    web::Json(case)
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/data", web::get().to(get_data))
                    .route("/{filename:.*}", web::get().to(index)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
