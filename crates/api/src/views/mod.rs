use actix_web::web::{get, scope, ServiceConfig};
mod ca;
use ca::parse_ca_info;

pub fn view_factory(app: &mut ServiceConfig) {
    app.service(scope("v1/ca").route("{ca}", get().to(parse_ca_info)));
}
