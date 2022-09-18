use std::sync::Arc;

use gtfs_structures::{Stop, Trip};

use crate::{multimap::MultiMap, types::RouteDir};

mod dag;

pub fn stops_by_route<'a>(
    trips: impl IntoIterator<Item = &'a Trip>,
    args: &crate::Args,
) -> anyhow::Result<MultiMap<RouteDir, Arc<Stop>>> {
    // first, collect trips by route id and direction
    let trips_by_route = trips
        .into_iter()
        .map(|trip| {
            (
                RouteDir::from_trip(trip, args.direction_from_trip_name),
                trip,
            )
        })
        .collect::<MultiMap<_, _>>();

    // then, merge all trips into a consolidated list of stops
    let mut stops_by_route = MultiMap::new();
    for (route, trips) in trips_by_route.map {
        let trips =
            merge_trips(trips).map_err(|err| anyhow::anyhow!("{err} in route {route:?}"))?;
        stops_by_route.insert_bulk(route, trips);
    }
    Ok(stops_by_route)
}

fn merge_trips(trips: Vec<&Trip>) -> anyhow::Result<Vec<Arc<Stop>>> {
    use dag::{Dag, PtrKey};
    // generate dag from trips
    let mut dag = Dag::<PtrKey<Stop>, Arc<Stop>>::new();
    for trip in trips {
        let mut parent = None;
        for st in &trip.stop_times {
            let child = st.stop.clone();
            dag.insert_child(parent, child.clone())?;
            parent = Some(child);
        }
    }
    dag.flatten()
}
