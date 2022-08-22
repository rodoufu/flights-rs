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

#[derive(Debug, Eq, PartialEq)]
pub enum FlightError {
    DestinationNotFoundFor(String),
    InvalidLeg(Vec<String>),
    InvalidAirportCode(String),
    SourceNotFound,
    DestinationNotFound,
}

pub trait FlightInfo {
    fn source(&self) -> String;
    fn destination(&self) -> String;
    fn path(&self) -> Result<Vec<String>, FlightError>;
}

/// Flight represents a graph with the sources and destinations of each leg.
/// It is caching the source and destination for better performance.
pub struct Flight {
    source: String,
    destination: String,
    from_to: HashMap<String, String>,
}

impl FlightInfo for Flight {
    fn source(&self) -> String {
        self.source.clone()
    }

    fn destination(&self) -> String {
        self.destination.clone()
    }

    fn path(&self) -> Result<Vec<String>, FlightError> {
        let mut path = Vec::new();
        path.push(self.source.clone());
        let mut current = self.source.clone();

        while current != self.destination {
            let dest = self.from_to.get(&current);
            if let Some(dest) = dest {
                path.push(dest.clone());
                current = dest.to_string();
            } else {
                return Err(FlightError::DestinationNotFoundFor(current.clone()));
            }
        }

        Ok(path)
    }
}
