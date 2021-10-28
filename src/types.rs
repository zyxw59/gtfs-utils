use gtfs_structures::DirectionType;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct RouteDir {
    pub route_id: String,
    pub direction: Direction,
}

impl RouteDir {
    pub fn new(route_id: String, direction: Option<DirectionType>) -> RouteDir {
        RouteDir {
            route_id,
            direction: direction.into(),
        }
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

impl From<Direction> for Option<DirectionType> {
    fn from(val: Direction) -> Option<DirectionType> {
        match val {
            Direction::None => None,
            Direction::Inbound => Some(DirectionType::Inbound),
            Direction::Outbound => Some(DirectionType::Outbound),
        }
    }
}
