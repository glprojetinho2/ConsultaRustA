use crate::erro;
use actix_web::{web, HttpRequest, Responder};
use serde_json::json;

pub async fn parse_ca_info(req: HttpRequest) -> impl Responder {
    let ca: u32 = match req.match_info().get("ca") {
        Some(c) => match c.parse() {
            Ok(v) => v,
            Err(_) => return web::Json(json!({"erro": erro!(1, c)})),
        },
        None => return web::Json(json!({"erro": erro!(3)})),
    };
    let pagina = cascraper::pagina(None, ca).await;
    let ca_info = match cascraper::CA::consultar(&pagina, ca).await {
        Ok(v) => v,
        Err(e) => match e {
            cascraper::errors::CAError::NaoEncontrado(ca) => {
                return web::Json(json!({"erro": erro!(2, ca)}))
            }
        },
    };
    web::Json(json!(ca_info))
}
