use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Flight {
    pub id: String,
    pub route: String,
    pub from: String,
    pub to: String,
    pub aircraft: String,
    pub price: f64,
    pub duration: String,
    pub seats: i32,
    pub departure_time: String,
}

pub struct FlightService;

impl FlightService {
    pub fn get_all_flights() -> Vec<Flight> {
        vec![
            Flight {
                id: Uuid::new_v4().to_string(),
                route: "New York → London".to_string(),
                from: "New York".to_string(),
                to: "London".to_string(),
                aircraft: "Gulfstream G650".to_string(),
                price: 125000.0,
                duration: "7h 30m".to_string(),
                seats: 14,
                departure_time: "2026-01-25T10:00:00Z".to_string(),
            },
            Flight {
                id: Uuid::new_v4().to_string(),
                route: "Los Angeles → Tokyo".to_string(),
                from: "Los Angeles".to_string(),
                to: "Tokyo".to_string(),
                aircraft: "Bombardier Global 7500".to_string(),
                price: 185000.0,
                duration: "11h 15m".to_string(),
                seats: 17,
                departure_time: "2026-01-26T14:00:00Z".to_string(),
            },
            Flight {
                id: Uuid::new_v4().to_string(),
                route: "Miami → Paris".to_string(),
                from: "Miami".to_string(),
                to: "Paris".to_string(),
                aircraft: "Dassault Falcon 8X".to_string(),
                price: 95000.0,
                duration: "8h 45m".to_string(),
                seats: 12,
                departure_time: "2026-01-27T09:00:00Z".to_string(),
            },
            Flight {
                id: Uuid::new_v4().to_string(),
                route: "Dubai → New York".to_string(),
                from: "Dubai".to_string(),
                to: "New York".to_string(),
                aircraft: "Gulfstream G700".to_string(),
                price: 210000.0,
                duration: "14h 20m".to_string(),
                seats: 19,
                departure_time: "2026-01-28T18:00:00Z".to_string(),
            },
            Flight {
                id: Uuid::new_v4().to_string(),
                route: "London → Dubai".to_string(),
                from: "London".to_string(),
                to: "Dubai".to_string(),
                aircraft: "Bombardier Global 6000".to_string(),
                price: 145000.0,
                duration: "6h 50m".to_string(),
                seats: 13,
                departure_time: "2026-01-29T12:00:00Z".to_string(),
            },
            Flight {
                id: Uuid::new_v4().to_string(),
                route: "Tokyo → Singapore".to_string(),
                from: "Tokyo".to_string(),
                to: "Singapore".to_string(),
                aircraft: "Cessna Citation X".to_string(),
                price: 78000.0,
                duration: "6h 30m".to_string(),
                seats: 8,
                departure_time: "2026-01-30T08:00:00Z".to_string(),
            },
        ]
    }

    pub fn search_flights(
        from: Option<&str>,
        to: Option<&str>,
        passengers: Option<i32>,
    ) -> Vec<Flight> {
        let all_flights = Self::get_all_flights();

        all_flights
            .into_iter()
            .filter(|flight| {
                let mut matches = true;

                if let Some(from_query) = from {
                    if !from_query.is_empty() {
                        matches = matches
                            && flight.from.to_lowercase().contains(&from_query.to_lowercase());
                    }
                }

                if let Some(to_query) = to {
                    if !to_query.is_empty() {
                        matches = matches
                            && flight.to.to_lowercase().contains(&to_query.to_lowercase());
                    }
                }

                if let Some(min_seats) = passengers {
                    matches = matches && flight.seats >= min_seats;
                }

                matches
            })
            .collect()
    }
}
