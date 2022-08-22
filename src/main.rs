mod flight;

use actix_web::{
    App,
    get,
    http::header::ContentType,
    HttpResponse,
    HttpServer,
    middleware::Logger,
    post,
    web::{
        self,
        Data,
    },
};
use actix_web_prom::PrometheusMetricsBuilder;
use crate::flight::{
    FlightRequest,
    FlightResponse,
};
use log::{
    error,
    info,
};
use prometheus::{
    opts,
    IntCounterVec,
};
use uuid::Uuid;

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/flight")]
async fn handle_flight(
    req: web::Json<FlightRequest>, counter: web::Data<IntCounterVec>,
) -> HttpResponse {
    let request_id = Uuid::new_v4();
    info!("got a request: {:?}", request_id);
    counter.with_label_values(&["endpoint", "method", "status"]).inc();

    let resp = match (&req.into_inner()).try_into() {
        Ok(resp) => resp,
        Err(err) => FlightResponse::Error {
            message: format!("{:?}", err),
        },
    };
    match serde_json::to_string(&resp) {
        Ok(body) => {
            match resp {
                FlightResponse::Ok { .. } => {
                    info!("finished request {:?} with success", request_id);
                    HttpResponse::Ok()
                        .content_type(ContentType::json()).body(body)
                },
                FlightResponse::Error { message } => {
                    info!("finished request {:?} with bad request: {}", request_id, message);
                    HttpResponse::BadRequest()
                        .content_type(ContentType::json()).body(body)
                }
            }
        }
        Err(err) => {
            error!("finished request {:?} with error {:?}", request_id, err);
            HttpResponse::InternalServerError()
                .content_type(ContentType::json())
                .body(format!("{{\"message\":{:?}}}", err))
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("starting application");
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    info!("using port: {}", port);
    let port = port.parse::<u16>().unwrap_or_else(|_| panic!("invalid port value: {}", port));

    let workers = std::env::var("WORKERS").unwrap_or_else(|_| "10".to_string());
    info!("using workers: {}", workers);
    let workers = workers.parse::<usize>()
        .unwrap_or_else(|_| panic!("invalid workers value: {}", port));

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
            .app_data(Data::new(flights_counter.clone()))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(handle_flight)
    ).workers(workers)
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
