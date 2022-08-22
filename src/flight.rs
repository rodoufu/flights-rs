use std::collections::HashMap;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Deserialize)]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<Vec<String>>,
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

impl TryFrom<&Vec<Vec<String>>> for Flight {
    type Error = FlightError;

    fn try_from(legs: &Vec<Vec<String>>) -> Result<Self, Self::Error> {
        let mut from_to = HashMap::new();
        let mut source_destination_code: HashMap<String, usize> = HashMap::new();
        for leg in legs {
            if leg.len() != 2 {
                return Err(FlightError::InvalidLeg(leg.clone()));
            }

            from_to.insert(leg[0].clone(), leg[1].clone());

            for (i, leg_it) in leg.iter().enumerate().take(2) {
                if leg_it.is_empty() {
                    return Err(FlightError::InvalidAirportCode(leg_it.clone()));
                }
                if source_destination_code.contains_key(leg_it) {
                    source_destination_code.remove(leg_it);
                } else {
                    source_destination_code.insert(leg_it.clone(), i);
                }
            }
        }

        let mut source = "".to_string();
        let mut destination = "".to_string();
        for (code, source_destination) in source_destination_code {
            if source_destination == 0 {
                source = code;
            } else {
                destination = code;
            }
        }

        if source.is_empty() {
            return Err(FlightError::SourceNotFound);
        }
        if destination.is_empty() {
            return Err(FlightError::DestinationNotFound);
        }

        Ok(Flight {
            source,
            destination,
            from_to,
        })
    }
}

impl TryFrom<&FlightRequest> for FlightResponse {
    type Error = FlightError;

    fn try_from(req: &FlightRequest) -> Result<Self, FlightError> {
        let flight: Flight = (&req.legs).try_into()?;
        let path = if req.full_path {
            Some(flight.path()?)
        } else {
            None
        };
        Ok(FlightResponse::Ok {
            source: flight.source(),
            destination: flight.destination(),
            path,
        })
    }
}
