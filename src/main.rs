use std::env;

use masslynx::reader::MassLynxReader;
use masslynx::{self, MassLynxError};

fn main() -> Result<(), MassLynxError> {
    pretty_env_logger::init_timed();
    let version = masslynx::get_mass_lynx_version();
    eprintln!("Using MassLynx Version: {:?}", version);
    let path = env::args().skip(1).next().unwrap();
    let spectrum_idx = env::args()
        .skip(2)
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_default();
    eprintln!("Opening {path}");
    let mut reader = MassLynxReader::from_path(&path)?;
    eprintln!("Opened reader with {} spectra", reader.len());

    let spec = reader.get_spectrum(spectrum_idx).unwrap();
    eprintln!("{:?}", spec);

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
    eprintln!(
        "TIC from {:0.2} to {:0.2} has maximum at {:0.2} with intensity {tic_max:0.2e}",
        tic_time.first().copied().unwrap_or_default(),
        tic_time.last().copied().unwrap_or_default(),
        tic_time.get(tic_max_idx).copied().unwrap_or_default()
    );
    Ok(())
}
