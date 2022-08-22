mod flight;

use actix_web::{
    App,
    HttpServer,
    middleware::Logger,
    post,
    web,
};
use crate::flight::{
    FlightRequest,
    FlightResponse,
};
use log::info;

#[post("/flight")]
async fn handle_flight(req: web::Json<FlightRequest>) -> web::Json<FlightResponse> {
    info!("got a new request");
    web::Json(FlightResponse::Error { message: "not implemented yet".to_string() })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("starting application");
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    info!("using port: {}", port);
    let port = port.parse::<u16>().expect(&format!("invalid port value: {}", port));

    HttpServer::new(
        || App::new().wrap(Logger::new("%a %{User-Agent}i")).service(handle_flight)
    ).workers(4)
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
