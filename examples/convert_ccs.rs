use std::{
    env,
    io::{self, prelude::*},
};

use csv;
use serde::{Serialize, Deserialize};

use masslynx::reader::MassLynxReader;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Record {
    mz: f32,
    charge: i32,
    drift_time: Option<f32>,
    ccs: Option<f32>,
}

fn read_records(query_path: &str) -> io::Result<Vec<Record>> {
    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_path(query_path)?;
    let records: Vec<_> = reader.deserialize().flatten().collect();
    return Ok(records);
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);

    let raw_path = args
        .next()
        .unwrap_or_else(|| panic!("Please provide a path to a Waters RAW directory"));

    let query_path = args
        .next()
        .unwrap_or_else(|| panic!("Please provide a path to a query CSV file"));

    let mut records = read_records(&query_path)?;

    let reader =
        MassLynxReader::from_path(raw_path).unwrap_or_else(|e| panic!("Failed to open RAW: {e}"));
    for rec in records.iter_mut() {
        if let Some(dt) = rec.drift_time {
            rec.ccs = Some(reader.get_ccs(dt, rec.mz, rec.charge).unwrap())
        } else if let Some(ccs) = rec.ccs {
            rec.drift_time = Some(
                reader
                    .get_drift_time_from_ccs(ccs, rec.mz, rec.charge)
                    .unwrap(),
            )
        }
    }

    let mut writer = io::stdout().lock();
    writer.write_all("mz,charge,drift_time,ccs\n\r".as_bytes())?;
    for rec in records {
        write!(
            writer,
            "{},{},{},{}\n\r",
            rec.mz,
            rec.charge,
            rec.drift_time.map(|v| v.to_string()).unwrap_or_default(),
            rec.ccs.map(|v| v.to_string()).unwrap_or_default()
        )?
    }

    Ok(())
}
