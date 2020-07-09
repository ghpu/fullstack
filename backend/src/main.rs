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
    let example_annot = common::Annotation{intent:"Music_play".to_string(), values:vec![
        ("artist".to_string(),"Joan Baez".to_string()),
         ("scene name".to_string(),"Joan Baez".to_string())
    ]};
    let example_annot2 = common::Annotation{intent:"Music_play".to_string(), values:vec![
        ("artist".to_string(),"John Baez".to_string()),
         ("scene name".to_string(),"John Baez".to_string())
    ]};
    let example_annot3 = common::Annotation{intent:"Movie_play".to_string(), values:vec![
        ("any".to_string(),"John Baez".to_string()),
        ("artist".to_string(),"John Baez".to_string()),
         ("scene name".to_string(),"John Baez".to_string())
    ]};


    let case = common::Case {reference:1, count:42,text:"Joue du Joan Baez".to_string(), gold:vec![example_annot.clone()], left:vec![example_annot2.clone()], right:vec![example_annot.clone(),example_annot3.clone()]};

    let corpus = std::fs::File::open("/home/ghpu/projets/rust/fullstack/result.json").expect("cannot open result.json");
    let json: serde_json::Value = serde_json::from_reader(corpus).expect("invalid json in result.json");
    let samples : &Vec<serde_json::Value> = json[0]["samples"].as_array().unwrap();
    let mut cases :Vec<common::Case>=vec![];
    for (i,s) in samples.iter().enumerate() {
            let annotation = common::Annotation{intent:s["intent"].as_str().unwrap().to_owned(), values:vec![]};
            cases.push(common::Case {reference: i, count:1, text:s["literal"].as_str().unwrap().to_owned(), gold:vec![annotation.clone()], left:vec![annotation.clone()], right:vec![annotation.clone()]});
        };

    let corpus = common::Corpus{intentMapping:common::IntentMapping{val:[("Music_play".to_string(),"Music".to_string()),("Movie_play".to_string(),"Television".to_string())].iter().cloned().collect()}, cases};




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
