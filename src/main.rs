use std::env::args;

use scraper::Html;

pub mod info;

#[tokio::main]
async fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    let ca = match args().nth(1) {
        Some(v) => match v.parse::<u32>() {
            Ok(v) => v,
            Err(_) => panic!("input não é numérico."),
        },
        None => panic!("digite um CA."),
    };
    let client = reqwest::Client::new();
    let resp = client
        .get("https://consultaca.com/".to_owned() + &ca.to_string())
        .send()
        .await;
    let body_txt = match resp {
        Ok(r) => match r.text().await {
            Ok(txt) => txt,
            Err(e) => panic!("{:#?}", e),
        },
        Err(e) => panic!("{}", e),
    };
    let body = Html::parse_document(&body_txt);
    let consulta = match info::CA::consultar(&body).await {
        Ok(c) => c,
        Err(e) => panic!("{:#?}", e),
    };
    println!("{:#?}", consulta);
}
