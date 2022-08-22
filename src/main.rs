mod flight;

use actix_web::{
    App,
    HttpServer,
    post,
    web,
};
use crate::flight::{
    FlightRequest,
    FlightResponse,
};

#[post("/flight")]
async fn handle_flight(req: web::Json<FlightRequest>) -> web::Json<FlightResponse> {
    web::Json(FlightResponse::Error { message: "not implemented yet".to_string() })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let port = port.parse::<u16>().expect(&format!("invalid port value: {}", port));

    HttpServer::new(|| App::new().service(handle_flight))
        .workers(4)
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
