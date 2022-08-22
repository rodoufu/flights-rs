use std::collections::HashMap;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Deserialize)]
pub struct FlightRequest {
    pub legs: Vec<Vec<String>>,
    #[serde(default)]
    pub full_path: bool,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum FlightResponse {
    Ok {
        source: String,
        destination: String,
        #[serde(default)]
        path: Vec<String>,
    },
    Error {
        message: String,
    },
}
