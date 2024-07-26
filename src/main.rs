use clap::{Parser, Subcommand};
use gtfs_structures::Gtfs;

mod bitvec;
mod merge;
mod multimap;
mod radius;
mod table;
mod types;

#[derive(Debug, Parser)]
pub struct Args {
    source: String,
    #[clap(subcommand)]
    command: Command,
    /// Only include routes with this `agency_id`
    #[clap(long)]
    agency: Option<String>,
    /// Only include routes with this `route_id`
    #[clap(long)]
    route: Option<String>,
    /// Use the `short_name` instead of `long_name` when displaying route names.
    #[clap(long)]
    use_short_name: bool,
    /// Use `trip_short_name` to determine direction: odd-numbered trips are outbound,
    /// even-numbered are inbound.
    #[clap(long)]
    direction_from_trip_name: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Produce a summary, in markdown format, listing each route/direction pair, and all stops
    /// served by trips on that route, in order.
    ///
    /// If route has multiple branches, the ordering between branches is unspecified.
    RouteSummary,
    /// Produce a set of tables in markdown format, one for each route/direction pair, showing all
    /// trips and their stop times at each stop on the route.
    TimeTable,
    /// Produce a set of tables in markdown format, one for each route/direction pair, showing all
    /// stopping patterns on the route.
    StoppingPatterns,
    /// Produce a list, in markdown format, listing each route/direction pair, and the radius and
    /// diameter of that route.
    RadiusDiameter,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    pretty_env_logger::init();

    let mut gtfs = Gtfs::new(&args.source)?;
    log_gtfs_info(&args.source, &gtfs);
    if let Some(route_id) = &args.route {
        if let Some((id, route)) = gtfs.routes.remove_entry(route_id) {
            gtfs.trips.retain(|_, trip| trip.route_id == id);
            gtfs.routes = [(id, route)].into_iter().collect();
        } else {
            anyhow::bail!("No route with id {route_id}");
        }
    }
    if let Some(agency) = &args.agency {
        gtfs.routes
            .retain(|_, route| route.agency_id.as_ref() == Some(agency));
        gtfs.trips
            .retain(|_, trip| gtfs.routes.contains_key(&trip.route_id));
    }

    match args.command {
        Command::RouteSummary => route_summary(gtfs, &args),
        Command::TimeTable => time_table(gtfs, &args),
        Command::StoppingPatterns => stopping_patterns(gtfs, &args),
        Command::RadiusDiameter => radius_and_diameter(gtfs, &args),
    }
}

fn route_summary(gtfs: Gtfs, args: &Args) -> anyhow::Result<()> {
    let stops_by_route = merge::stops_by_route(gtfs.trips.values(), args)?;

    for (route, stops) in stops_by_route.map {
        println!("## {}", route.format(args.use_short_name, &gtfs.routes));
        for stop in stops {
            println!("- {}", stop.name);
        }
        println!();
    }
    Ok(())
}

fn time_table(gtfs: Gtfs, args: &Args) -> anyhow::Result<()> {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::table::{Align, Table};

    let stops_by_route = merge::stops_by_route(gtfs.trips.values(), args)?;

    let mut tables = BTreeMap::new();

    for trip in gtfs.trips.values() {
        let route_dir = types::RouteDir::from_trip(trip, args.direction_from_trip_name);
        let stops = stops_by_route
            .map
            .get(&route_dir)
            .expect("missing route/dir");
        let table = tables
            .entry(route_dir)
            .or_insert_with(|| Table::new(stops.clone()));
        let column = table.add_column(
            trip.trip_short_name
                .clone()
                .unwrap_or_else(|| trip.id.clone()),
            None,
        );

        // step thru `stop.times` one at a time. since they are already sorted, we can linearly
        // search thru `stops` for a match.
        let mut stops = stops.iter().zip(column.iter_mut());
        for stop_time in &trip.stop_times {
            if let Some((_, cell)) = stops.find(|(stop, _)| Arc::ptr_eq(stop, &stop_time.stop)) {
                *cell = stop_time.arrival_time.or(stop_time.departure_time);
            } else {
                log::error!("couldn't find stop {}", stop_time.stop);
                break;
            }
        }
    }

    for (route, table) in tables {
        println!("## {}", route.format(args.use_short_name, &gtfs.routes));
        println!();

        println!(
            "{}",
            table.formatter(
                |trip_name| trip_name,
                |stop| &stop.name,
                |time| format_time_optional(*time),
                Align::Right,
            )
        );
    }
    println!();

    Ok(())
}

fn stopping_patterns(gtfs: Gtfs, args: &Args) -> anyhow::Result<()> {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::{
        bitvec::BitVec,
        table::{Align, Table},
    };

    let mut stops_by_route = merge::stops_by_route(gtfs.trips.values(), args)?;

    let mut patterns_by_route = BTreeMap::new();

    for trip in gtfs.trips.values() {
        let route_dir = types::RouteDir::from_trip(trip, args.direction_from_trip_name);
        let stops = stops_by_route
            .map
            .get(&route_dir)
            .expect("missing route/dir");

        let patterns = patterns_by_route
            .entry(route_dir)
            .or_insert_with(BTreeMap::new);
        let mut pattern = BitVec::with_size(stops.len());

        // step thru `stop.times` one at a time. since they are already sorted, we can linearly
        // search thru `stops` for a match.
        let mut stops = stops.iter().enumerate();
        for stop_time in &trip.stop_times {
            if let Some((i, _)) = stops.find(|(_, stop)| Arc::ptr_eq(stop, &stop_time.stop)) {
                pattern.set(i);
            }
        }
        *patterns.entry(pattern).or_insert(0) += 1;
    }

    for (route_dir, patterns) in patterns_by_route {
        let stops = stops_by_route
            .map
            .remove(&route_dir)
            .expect("missing route/dir");
        let mut table = Table::new(stops);
        for (pattern, count) in patterns {
            table.push_column(count, pattern.to_vec())?;
        }
        println!("## {}", route_dir.format(args.use_short_name, &gtfs.routes));
        println!();

        println!(
            "{}",
            table.formatter(
                |count| count,
                |stop| &stop.name,
                |&does_stop| if does_stop { "•" } else { "" },
                Align::Center,
            )
        );
    }
    println!();

    Ok(())
}

fn radius_and_diameter(gtfs: Gtfs, args: &Args) -> anyhow::Result<()> {
    let stops_by_route = merge::stops_by_route_unsorted(gtfs.trips.values(), args)?;

    let rds = stops_by_route.map.into_iter().map(|(k, v)| {
        let points = v.into_iter().filter_map(|stop| {
            stop.longitude
                .and_then(|long| stop.latitude.map(|lat| geo::Point::new(long, lat)))
        }).collect::<Vec<_>>();
        let r_d = radius::radius_and_diameter(&points);
        (k, r_d)
    }).collect::<Vec<_>>();

    println!("Route | radius | diameter");
    println!("--- | --- | ---");
    for (route, (radius, diameter)) in rds {
        println!("{} | {radius:.3} | {diameter:.3}", route.format(args.use_short_name, &gtfs.routes));
    }

    Ok(())
}

fn format_time_optional(time: Option<u32>) -> std::borrow::Cow<'static, str> {
    use std::borrow::Cow;

    if let Some(time) = time {
        let hrs = time / (60 * 60);
        let mins = (time / 60) % 60;
        let secs = time % 60;
        Cow::Owned(format!("{:02}:{:02}:{:02}", hrs, mins, secs))
    } else {
        Cow::Borrowed("")
    }
}

fn log_gtfs_info(source: &str, gtfs: &Gtfs) {
    log::info!("Loaded GTFS data from {}:", source);
    log::info!("  Read in {} ms", gtfs.read_duration);
    log::info!("  Stops: {}", gtfs.stops.len());
    log::info!("  Routes: {}", gtfs.routes.len());
    log::info!("  Trips: {}", gtfs.trips.len());
    log::info!("  Agencies: {}", gtfs.agencies.len());
    log::info!("  Shapes: {}", gtfs.shapes.len());
    log::info!("  Fare attributes: {}", gtfs.fare_attributes.len());
    log::info!("  Feed info: {}", gtfs.feed_info.len());
}
