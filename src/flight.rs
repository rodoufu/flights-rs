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
        let mut path = vec![self.source.clone()];
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

mod test {
    use std::collections::HashMap;
    use crate::flight::{
        Flight,
        FlightError,
        FlightInfo,
    };

    #[test]
    fn should_get_destination_not_found_for() {
        // Given
        let mut from_to: HashMap<String, String> = HashMap::new();
        from_to.insert("SFO".to_string(), "IND".to_string());
        let flight = Flight {
            source: "SFO".to_string(),
            destination: "EWR".to_string(),
            from_to,
        };

        // When
        let path = flight.path();

        // Then
        assert!(path.is_err());
        let path = path.err().unwrap();
        assert_eq!(FlightError::DestinationNotFoundFor("IND".to_string()), path);
    }

    #[test]
    fn should_get_invalid_leg() {
        // Given
        let legs = vec![
            vec!["SFO".to_string(), "EWR".to_string(), "SFO".to_string(), "EWR".to_string()],
        ];

        // When
        let flight: Result<Flight, FlightError> = (&legs).try_into();

        // Then
        assert!(flight.is_err());
        assert_eq!(FlightError::InvalidLeg(legs[0].clone()), flight.err().unwrap());
    }

    #[test]
    fn should_get_invalid_airport_code() {
        // Given
        let legs = vec![
            vec!["SFO".to_string(), "".to_string()],
        ];

        // When
        let flight: Result<Flight, FlightError> = (&legs).try_into();

        // Then
        assert!(flight.is_err());
        assert_eq!(FlightError::InvalidAirportCode("".to_string()), flight.err().unwrap());
    }

    #[test]
    fn should_get_destination_not_found() {
        // Given
        let legs = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["B".to_string(), "C".to_string()],
            vec!["B".to_string(), "C".to_string()],
        ];

        // When
        let flight: Result<Flight, FlightError> = (&legs).try_into();

        // Then
        assert!(flight.is_err());
        assert_eq!(FlightError::DestinationNotFound, flight.err().unwrap());
    }

    #[test]
    fn should_not_get_a_flight() {
        // Given
        let legs = Vec::new();

        // When
        let flight: Result<Flight, FlightError> = (&legs).try_into();

        // Then
        assert!(flight.is_err());
        assert_eq!(FlightError::SourceNotFound, flight.err().unwrap());
    }

    #[test]
    fn should_get_a_flight_with_one_leg() {
        // Given
        let legs = vec![
            vec!["SFO".to_string(), "EWR".to_string()],
        ];

        // When
        let flight: Result<Flight, FlightError> = (&legs).try_into();

        // Then
        assert!(flight.is_ok());
        let flight = flight.ok().unwrap();
        assert_eq!("SFO".to_string(), flight.source());
        assert_eq!("EWR".to_string(), flight.destination());

        let path = flight.path();
        assert!(path.is_ok());
        let path = path.ok().unwrap();
        assert_eq!(vec!["SFO".to_string(), "EWR".to_string()], path);
    }

    #[test]
    fn should_get_a_flight_with_two_legs() {
        // Given
        let legs = vec![
            vec!["SFO".to_string(), "EWR".to_string()],
            vec!["EWR".to_string(), "IND".to_string()],
        ];

        // When
        let flight: Result<Flight, FlightError> = (&legs).try_into();

        // Then
        assert!(flight.is_ok());
        let flight = flight.ok().unwrap();
        assert_eq!("SFO".to_string(), flight.source());
        assert_eq!("IND".to_string(), flight.destination());

        let path = flight.path();
        assert!(path.is_ok());
        let path = path.ok().unwrap();
        assert_eq!(vec!["SFO".to_string(), "EWR".to_string(), "IND".to_string()], path);
    }
}
