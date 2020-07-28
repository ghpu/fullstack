use actix::{Actor, StreamHandler};
use actix_files::NamedFile;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;
use futures::future;
use std::path::PathBuf;

struct MyWs;
impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    println!("{:?}", resp);
    resp
}

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut path: PathBuf = req.match_info().query("filename").parse().unwrap();
    if path.exists() {
        Ok(NamedFile::open(path)?)
    } else {
        Ok(NamedFile::open("index.html")?)
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let http_server =
        HttpServer::new(move || App::new().route("/{filename:.*}", web::get().to(index)))
            .bind("0.0.0.0:8080")?
            .run();
    let ws_server = HttpServer::new(move || App::new().route("/", web::get().to(ws)))
        .bind("0.0.0.0:9001")?
        .run();
    future::try_join(http_server, ws_server).await?;
    Ok(())
}
