use std::env;
use masslynx::reader::MassLynxReader;
use masslynx::{self, MassLynxError, MassLynxResult};

#[allow(unused)]
fn show_spectrum(reader: &mut MassLynxReader) {
    let spectrum_idx = env::args()
        .skip(2)
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_default();

    // This may panic if the index is out of bounds
    let spec = match reader.get_spectrum(spectrum_idx) {
        Some(s) => s,
        None => panic!("Index {} out of bounds for file {:?} with {} spectra", spectrum_idx, reader.path(), reader.len()),
    };
    eprintln!("{:?}", spec);
}

#[allow(unused)]
fn show_cycle(reader: &mut MassLynxReader) {
    let spectrum_idx = env::args()
        .skip(2)
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_default();

    // This may panic if the index is out of bounds
    match reader.get_cycle(spectrum_idx) {
        Some(spec) => {
            eprintln!("{:?}", spec);
        },
        None => {
            match reader.cycle_index().get(spectrum_idx) {
                Some(c) => {
                    if !c.has_drift_time() {
                        eprintln!("Cycle {spectrum_idx} has no ion mobility");
                    } else {
                        panic!("Index {} out of bounds for file {:?} with {} cycles", spectrum_idx, reader.path(), reader.cycle_index().len())
                    }
                },
                None => panic!("Index {} out of bounds for file {:?} with {} cycles", spectrum_idx, reader.path(), reader.cycle_index().len())
            }
        },
    };
}

#[allow(unused)]
fn show_chromatogram(reader: &mut MassLynxReader) {
    let mass = env::args()
        .skip(2)
        .next()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(366.14);

    let (time, ints) = reader.read_xic(0, mass, 0.2, false).unwrap();

    time.into_iter().zip(ints).for_each(|(t, i)| {
        eprintln!("{t}\t{i}");
    });
}

fn show_ms_level_counts(reader: &mut MassLynxReader) {
    reader.set_signal_loading(false);
    let mut counters = [0, 0, 0];
    let funcs = reader.functions().to_vec();
    for cycle in reader.iter_cycles() {
        counters[funcs[cycle.function()].ms_level as usize] += 1;
    }

    eprintln!("MS Levels: {counters:?}");
}

fn show_tic(reader: &mut MassLynxReader) -> MassLynxResult<()> {
    let (tic_time, tic_int) = reader.tic()?;
    let (tic_max_idx, tic_max) = tic_int
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap_or_default();

    eprintln!(
        "TIC from {:0.2} to {:0.2} has maximum at {:0.2} with intensity {tic_max:0.2e}",
        tic_time.first().copied().unwrap_or_default(),
        tic_time.last().copied().unwrap_or_default(),
        tic_time.get(tic_max_idx).copied().unwrap_or_default()
    );
    Ok(())
}

#[allow(unused)]
fn show_analog(reader: &mut MassLynxReader) -> MassLynxResult<()> {
    for trace in reader.iter_analogs() {
        eprintln!("{} | {} | {:?}: {}", trace.name, trace.unit, trace.unit.as_bytes(), trace.time.len());
    }
    Ok(())
}

fn show_mobilogram(reader: &mut MassLynxReader) -> MassLynxResult<()> {
    let (time_array, intensity_array) = reader.read_mobilogram(
        0, 0, 10, 50.0, 200.0)?;
    eprintln!("Mobilogram from {:0.2} to {:0.2} with intensity range from {:0.2} to {:0.2},
    ",
    time_array.first().copied().unwrap_or_default(),
    time_array.last().copied().unwrap_or_default(),
    intensity_array.first().copied().unwrap_or_default(),
    intensity_array.last().copied().unwrap_or_default()
);
    Ok(())
}

fn main() -> Result<(), MassLynxError> {
    pretty_env_logger::init_timed();
    let version = masslynx::get_mass_lynx_version();
    eprintln!("Using MassLynx Version: {:?}", version);
    let path = env::args().skip(1).next().unwrap();

    eprintln!("Opening {path}");
    let mut reader = MassLynxReader::from_path(&path)?;
    eprintln!("Opened reader with {} spectra", reader.len());

    eprintln!("{:?}", reader.header_items().unwrap());
    eprintln!("{:?}", reader.acquisition_information().unwrap());
    show_ms_level_counts(&mut reader);
    show_analog(&mut reader)?;
    // show_spectrum(&mut reader);
    // show_cycle(&mut reader);
    show_chromatogram(&mut reader);
    if let Err(e) = show_mobilogram(&mut reader) {
        eprintln!("No mobilogram read: {e}");
    }
    show_tic(&mut reader)?;
    Ok(())
}
