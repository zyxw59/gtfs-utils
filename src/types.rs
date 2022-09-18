use gtfs_structures::{DirectionType, Route, Trip};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct RouteDir {
    pub route_id: String,
    pub direction: Direction,
}

impl RouteDir {
    pub fn from_trip(trip: &Trip, direction_from_trip_name: bool) -> RouteDir {
        let direction = if direction_from_trip_name {
            trip.trip_short_name
                .as_ref()
                .and_then(|name| name.parse::<u32>().ok())
                .into()
        } else {
            trip.direction_id.into()
        };
        RouteDir {
            route_id: trip.route_id.clone(),
            direction,
        }
    }

    pub fn format(&self, routes: &std::collections::HashMap<String, Route>) -> String {
        let route_name = routes
            .get(&self.route_id)
            .map(|r| &r.long_name)
            .unwrap_or(&self.route_id);
        format!("{} ({:?})", route_name, self.direction)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Direction {
    None,
    Inbound,
    Outbound,
}

impl From<Option<DirectionType>> for Direction {
    fn from(val: Option<DirectionType>) -> Direction {
        match val {
            None => Direction::None,
            Some(DirectionType::Inbound) => Direction::Inbound,
            Some(DirectionType::Outbound) => Direction::Outbound,
        }
    }
}

impl From<Option<u32>> for Direction {
    fn from(val: Option<u32>) -> Direction {
        match val {
            None => Direction::None,
            Some(x) if x % 2 == 0 => Direction::Inbound,
            Some(_) => Direction::Outbound,
        }
    }
}

impl From<Direction> for Option<DirectionType> {
    fn from(val: Direction) -> Option<DirectionType> {
        match val {
            Direction::None => None,
            Direction::Inbound => Some(DirectionType::Inbound),
            Direction::Outbound => Some(DirectionType::Outbound),
        }
    }
}
