use crate::erro;
use actix_web::HttpRequest;
use serde_json::json;

pub async fn parse_ca_info(req: HttpRequest) -> String {
    let ca: u32 = match req.match_info().get("ca") {
        Some(c) => match c.parse() {
            Ok(v) => v,
            Err(_) => return json!({"erro": erro!(1, c)}).to_string(),
        },
        None => return json!({"erro": erro!(3)}).to_string(),
    };
    let pagina = cascraper::pagina(None, ca).await;
    let ca_info = match cascraper::CA::consultar(&pagina, ca).await {
        Ok(v) => v,
        Err(e) => match e {
            cascraper::errors::CAError::NaoEncontrado(ca) => {
                return json!({"erro": erro!(2, ca)}).to_string()
            }
        },
    };
    serde_json::to_string(&ca_info).unwrap()
}
