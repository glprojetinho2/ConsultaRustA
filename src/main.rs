use std::env::args;

use scraper::{Html, Selector};

mod info;

#[tokio::main]
async fn main() {
    let ca = match args().skip(1).next() {
        Some(v) => match v.parse::<u32>() {
            Ok(v) => v,
            Err(_) => panic!("input não é numérico.")
        },
        None => panic!("digite um CA.")
    };
    let client = reqwest::Client::new();
    let consulta = match info::CA::consultar(ca,client).await {
        Ok(c) => c,
        Err(e) => panic!("{:#?}", e)

    };
    println!("{:#?}",consulta);
}


