use gtfs_structures::Gtfs;
use structopt::StructOpt;

mod merge;
mod multimap;
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
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    pretty_env_logger::init();

    let gtfs = Gtfs::new(&args.source)?;
    log_gtfs_info(&args.source, &gtfs);

    match args.command {
        Command::RouteSummary => route_summary(gtfs),
    }
}

fn route_summary(gtfs: Gtfs) -> anyhow::Result<()> {
    let stops_by_route = merge::stops_by_route(gtfs.trips.values())?;

    for (route, stops) in stops_by_route.map {
        let route_name = gtfs
            .routes
            .get(&route.route_id)
            .map(|r| &r.long_name)
            .unwrap_or(&route.route_id);
        println!("## {} ({:?})", route_name, route.direction);
        for stop in stops {
            println!("- {}", stop.name);
        }
        println!();
    }
    Ok(())
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
