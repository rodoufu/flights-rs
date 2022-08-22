mod flight;

use actix_web::{
    App,
    get,
    HttpResponse,
    HttpServer,
    middleware::Logger,
    post,
    web,
};
use actix_web_prom::{
    PrometheusMetrics,
    PrometheusMetricsBuilder,
};
use crate::flight::{
    FlightRequest,
    FlightResponse,
};
use log::info;
use prometheus::{
    opts,
    IntCounterVec,
};
use std::collections::HashMap;

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/flight")]
async fn handle_flight(
    req: web::Json<FlightRequest>, counter: web::Data<IntCounterVec>,
) -> web::Json<FlightResponse> {
    info!("got a new request");
    counter.with_label_values(&["endpoint", "method", "status"]).inc();
    web::Json(FlightResponse::Error { message: "not implemented yet".to_string() })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("starting application");
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    info!("using port: {}", port);
    let port = port.parse::<u16>().expect(&format!("invalid port value: {}", port));

    let flights_counter_opts = opts!("flight_requests", "number of flight requests")
        .namespace("flights");
    let flights_counter = IntCounterVec::new(
        flights_counter_opts, &["endpoint", "method", "status"],
    ).expect("problem creating flights counter");

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .expect("problem creating prometheus builder");

    prometheus
        .registry
        .register(Box::new(flights_counter.clone()))
        .unwrap();

    HttpServer::new(move ||
        App::new()
            .wrap(prometheus.clone())
            .data(flights_counter.clone())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(handle_flight)
    ).workers(4)
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
