use gtfs_structures::Gtfs;
use structopt::StructOpt;

mod merge;
mod multimap;
mod table;
mod types;

#[derive(Debug, StructOpt)]
struct Args {
    source: String,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Produce a summary, in markdown format, listing each route/direction pair, and all stops
    /// served by trips on that route, in order.
    ///
    /// If route has multiple branches, the ordering between branches is unspecified.
    RouteSummary,
    /// Produce a set of tables in markdown format, one for each route/direction pair, showing all
    /// trips and their stop times at each stop on the route.
    TimeTable,
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    pretty_env_logger::init();

    let gtfs = Gtfs::new(&args.source)?;
    log_gtfs_info(&args.source, &gtfs);

    match args.command {
        Command::RouteSummary => route_summary(gtfs),
        Command::TimeTable => time_table(gtfs),
    }
}

fn route_summary(gtfs: Gtfs) -> anyhow::Result<()> {
    let stops_by_route = merge::stops_by_route(gtfs.trips.values())?;

    for (route, stops) in stops_by_route.map {
        println!("## {}", route.format(&gtfs.routes));
        for stop in stops {
            println!("- {}", stop.name);
        }
        println!();
    }
    Ok(())
}

fn time_table(gtfs: Gtfs) -> anyhow::Result<()> {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::table::Table;

    let stops_by_route = merge::stops_by_route(gtfs.trips.values())?;

    let mut tables = BTreeMap::new();

    for trip in gtfs.trips.values() {
        let route_dir = types::RouteDir::new(trip.route_id.clone(), trip.direction_id);
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
            }
        }
    }

    for (route, table) in tables {
        println!("## {}", route.format(&gtfs.routes));
        println!();

        print!("—");
        for header in table.col_headers() {
            print!("| {}", header);
        }
        println!();

        print!("---");
        for _ in table.col_headers() {
            print!("|---");
        }
        println!();

        for (stop, row) in table.row_headers().iter().zip(table.rows()) {
            print!("**{}**", stop.name);
            for time in row {
                print!(" | {}", format_time_optional(*time));
            }
            println!();
        }
    }

    println!();

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
