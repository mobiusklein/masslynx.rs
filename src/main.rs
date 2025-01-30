use std::env;

use log::info;

use masslynx::reader::MassLynxReader;
use masslynx::{self, MassLynxError};

fn main() -> Result<(), MassLynxError> {
    pretty_env_logger::init_timed();
    let version = masslynx::get_mass_lynx_version();
    info!("Using MassLynx Version: {:?}", version);
    let path = env::args().skip(1).next().unwrap();
    info!("Opening {path}");
    let mut reader = MassLynxReader::from_path(&path)?;
    info!("Opened reader with {} spectra", reader.len());

    let mut iter = reader.iter_spectra();
    let spec = iter.next().unwrap();
    drop(iter);

    info!("{:?}", spec);

    let (tic_time, tic_int) = reader.tic()?;
    let (tic_max_idx, tic_max) = tic_int
        .iter()
        .copied()
        .enumerate()
        .reduce(
            |(i, max), (j, next)| {
                if max > next {
                    (i, max)
                } else {
                    (j, next)
                }
            },
        )
        .unwrap_or_default();
    info!(
        "TIC from {:0.2} to {:0.2} has maximum at {:0.2} with intensity {tic_max:0.2e}",
        tic_time.first().copied().unwrap_or_default(),
        tic_time.last().copied().unwrap_or_default(),
        tic_time.get(tic_max_idx).copied().unwrap_or_default()
    );
    Ok(())
}
