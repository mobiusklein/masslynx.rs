use std::{
    io,
    path::PathBuf,
};

use clap::Parser;
use csv;
use serde::{Deserialize, Serialize};

use masslynx::reader::MassLynxReader;
use mzpeaks::{coordinate::SimpleInterval, prelude::Span1D, CoordinateRange};


/// A simple tool to read Waters RAW files and extract a 4D slice of data.
///
/// Data will be written to `STDOUT` as a CSV. Use stream redirection `>` to capture
/// in a file or pipe into another program.
#[derive(Parser)]
struct App {
    /// The path to the Waters RAW directory
    #[arg()]
    raw_path: PathBuf,
    /// The span of time within which to extract signal.
    ///
    /// Uses the notation (start?)-(end?) where either side may be omitted and
    /// and will be set to 0 and infinity respectively.
    #[arg()]
    time_range: CoordinateRange<f64>,

    /// The m/z range to extract signal for. If omitted, the entire mass range will be extracted.
    ///
    /// Uses the same notation as `TIME_RANGE`
    #[arg(short, long)]
    mz_range: Option<CoordinateRange<f64>>,

    /// The drift time range to extract signal for. If omitted, the entire drift time range will be extracted
    ///
    /// Uses the same notation as `TIME_RANGE`
    #[arg(short, long)]
    dt_range: Option<CoordinateRange<f64>>,

    /// The lock mass to use to correct the reported m/z values
    ///
    /// If omitted, the uncalibrated m/z values will be reported.
    #[arg(short, long)]
    lockmass: Option<f32>
}

#[derive(Debug, Serialize, Deserialize)]
struct ResultRecord {
    function: usize,
    cycle_index: usize,
    cycle_time: f64,
    drift_scan_index: usize,
    drift_time: f64,
    mz: f64,
    intensity: f32,
}

impl ResultRecord {
    fn new(function: usize, cycle_index: usize, cycle_time: f64, drift_scan_index: usize, drift_time: f64, mz: f64, intensity: f32) -> Self {
        Self { function, cycle_index, cycle_time, drift_scan_index, drift_time, mz, intensity }
    }


}

fn main() -> io::Result<()> {
    pretty_env_logger::formatted_timed_builder().parse_default_env().parse_env("WATERS_RAW_SLICE_LOG").init();
    let args = App::parse();
    log::info!("Opening {}", args.raw_path.display());
    let mut reader = MassLynxReader::from_path(&args.raw_path).unwrap();

    if let Some(lock_mass) = args.lockmass {
        reader.set_lock_mass(lock_mass, None).unwrap_or_else(|e| panic!("Failed to set lock mass: {e}"));
    }
    // reader.set_lockmass_skipping(true);

    let time_range = SimpleInterval::new(
        args.time_range.start().unwrap_or_default(),
        args.time_range
            .end()
            .unwrap_or(reader.cycle_index().last().unwrap().time),
    );

    let dt_range = match args.dt_range {
        Some(dt) => SimpleInterval::new(
            dt.start().unwrap_or_default(),
            dt.end().unwrap_or(f64::INFINITY),
        ),
        None => SimpleInterval::new(0.0, f64::INFINITY),
    };

    let mz_range = match args.mz_range {
        Some(mz) => SimpleInterval::new(
            mz.start().unwrap_or_default(),
            mz.end().unwrap_or(f64::INFINITY),
        ),
        None => SimpleInterval::new(0.0, f64::INFINITY),
    };

    let writer = io::stdout().lock();
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for i in 0..reader.cycle_index().len() {
        let cycle = reader.cycle_index()[i];
        if time_range.contains(&cycle.time) {
            if i % 100 == 0 {
                log::info!("Reading {cycle:?}");
            }
            let index = cycle.index;
            let cycle = match reader.get_cycle(index) {
                Some(cycle) => cycle,
                None => {
                    continue;
                }
            };
            for (i, dt_scan) in cycle.signal.iter().enumerate() {
                if dt_range.contains(&dt_scan.drift_time) {
                    for (mz, inten) in dt_scan
                        .mz_array
                        .iter()
                        .copied()
                        .zip(dt_scan.intensity_array.iter().copied())
                    {
                        let mz = mz as f64;
                        if mz_range.contains(&mz) {
                            let rec = ResultRecord::new(
                                cycle.function() + 1,
                                cycle.index + 1,
                                cycle.time,
                                i + 1,
                                dt_scan.drift_time,
                                mz,
                                inten,
                            );
                            writer.serialize(rec)?
                        }
                    }
                }
            }
        }
        if time_range.end < cycle.time {
            break;
        }
    }



    Ok(())
}
