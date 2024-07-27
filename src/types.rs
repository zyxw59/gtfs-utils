use gtfs_structures::{DirectionType, Route, Trip};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct RouteDir {
    pub route_id: Option<String>,
    pub direction: Direction,
}

impl RouteDir {
    pub fn from_trip(trip: &Trip, args: &crate::Args) -> RouteDir {
        let route_id = if args.merge_routes {
            None
        } else {
            Some(trip.route_id.clone())
        };
        RouteDir {
            route_id,
            direction: Direction::from_trip(trip, args.direction_from_trip_name),
        }
    }

    pub fn format(
        &self,
        use_short_name: bool,
        routes: &std::collections::HashMap<String, Route>,
    ) -> String {
        if let Some(route_id) = &self.route_id {
            let route_name = routes
                .get(route_id)
                .map(|r| {
                    if use_short_name {
                        &r.short_name
                    } else {
                        &r.long_name
                    }
                })
                .filter(|name| !name.is_empty())
                .unwrap_or(route_id);
            format!("{route_name} ({:?})", self.direction)
        } else {
            format!("{:?}", self.direction)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Direction {
    None,
    Inbound,
    Outbound,
}

impl Direction {
    pub fn from_trip(trip: &Trip, direction_from_trip_name: bool) -> Self {
        if direction_from_trip_name {
            trip.trip_short_name
                .as_ref()
                .and_then(|name| name.parse::<u32>().ok())
                .into()
        } else {
            trip.direction_id.into()
        }
    }
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
