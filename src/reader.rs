//! The higher-ish level API

use std::path::{Path, PathBuf};

use crate::{
    base::MassLynxChromatogramReader, AsMassLynxSource,
    constants::{LockMassParameter, MassLynxFunctionType, MassLynxIonMode},
    MassLynxInfoReader, MassLynxLockMassProcessor, MassLynxParameters,
    MassLynxResult, MassLynxScanReader,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpectrumIndexEntry {
    pub function: usize,
    pub cycle: usize,
    pub drift_index: Option<u32>,
}

impl SpectrumIndexEntry {
    pub fn new(function: usize, cycle: usize, drift_index: Option<u32>) -> Self {
        Self {
            function,
            cycle,
            drift_index,
        }
    }

    pub fn has_drift_time(&self) -> bool {
        self.drift_index.is_some()
    }

    pub fn native_id(&self) -> String {
        let i = match self.drift_index {
            Some(i) => i as usize,
            None => self.cycle,
        };
        format!("function={} process=0 scan={}", self.function + 1, i + 1)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CycleIndexEntry {
    pub function: usize,
    pub block: usize,
    pub time: f64,
    pub im_block_size: usize,
    pub index: usize,
}

impl CycleIndexEntry {
    pub fn new(
        function: usize,
        block: usize,
        time: f64,
        im_block_size: usize,
        index: usize,
    ) -> Self {
        Self {
            function,
            block,
            time,
            im_block_size,
            index,
        }
    }

    pub fn has_drift_time(&self) -> bool {
        self.im_block_size > 0
    }

    pub fn native_id(&self) -> String {
        if self.has_drift_time() {
            format!(
                "function={} process=0 startScan={} endScan={}",
                self.function + 1,
                self.im_block_size * self.index,
                self.im_block_size * self.index + self.im_block_size,
            )
        } else {
            format!(
                "function={} process=0 scan={}",
                self.function + 1,
                self.index + 1,
            )
        }
    }
}

pub struct MassLynxReader {
    path: PathBuf,
    scan_reader: MassLynxScanReader,
    info_reader: MassLynxInfoReader,
    chromatogram_reader: MassLynxChromatogramReader,
    lockmass_processor: MassLynxLockMassProcessor,
    cycle_index: Vec<CycleIndexEntry>,
    spectrum_index: Vec<SpectrumIndexEntry>,
}

impl MassLynxReader {
    pub fn from_path(path: &str) -> MassLynxResult<Self> {
        let info_reader = MassLynxInfoReader::from_path(&path)?;
        let scan_reader = MassLynxScanReader::from_source(&info_reader)?;
        let chromatogram_reader = MassLynxChromatogramReader::from_source(&info_reader)?;
        let mut lockmass_processor = MassLynxLockMassProcessor::new()?;
        lockmass_processor.set_raw_data_from_reader(&scan_reader)?;
        let path = PathBuf::from(path);

        let mut this = Self {
            path,
            info_reader,
            scan_reader,
            chromatogram_reader,
            lockmass_processor,
            cycle_index: Default::default(),
            spectrum_index: Default::default(),
        };

        this.build_index()?;
        Ok(this)
    }

    pub fn get_lock_mass_function(&self) -> Option<usize> {
        self.info_reader.get_lock_mass_function().ok().map(|(_, func)| func)
    }

    pub fn set_lock_mass(&mut self, mass: f32, tolerance: Option<f32>) -> MassLynxResult<()> {
        let mut params = MassLynxParameters::new()?;

        params.set(LockMassParameter::MASS, mass.to_string())?;

        match tolerance {
            Some(val) => {
                params.set(LockMassParameter::TOLERANCE, val.to_string())?;
            }
            None => {
                params.set(LockMassParameter::TOLERANCE, "0.25".to_string())?;
            }
        }

        self.lockmass_processor.set_parameters(&params)?;

        if self.lockmass_processor.can_lock_mass_correct()? {
            self.lockmass_processor.lock_mass_correct()?;
        }
        Ok(())
    }

    fn translate_function_type_to_ms_level(&mut self, fnum: usize) -> MassLynxResult<u8> {
        let ftype = self.info_reader.get_function_type(fnum)?;
        match ftype {
            MassLynxFunctionType::MS
            | MassLynxFunctionType::TOF
            | MassLynxFunctionType::TOFM
            | MassLynxFunctionType::PAR
            | MassLynxFunctionType::MTOF
            | MassLynxFunctionType::TOFP => Ok(1),
            MassLynxFunctionType::MS2 | MassLynxFunctionType::TOFD | MassLynxFunctionType::DAU => {
                Ok(2)
            }
            _ => Ok(0),
        }
    }

    fn build_index(&mut self) -> MassLynxResult<()> {
        let mut cycle_index = Vec::new();
        let n_funcs = self.info_reader.function_count()?;

        for func in 0..n_funcs {
            let ms_level = self.translate_function_type_to_ms_level(func)?;
            if ms_level == 0 {
                continue;
            }
            let scan_count = self.info_reader.scan_count_for_function(func)?;
            let im_block_size = self.info_reader.get_drift_scan_count(func)?;
            for i in 0..scan_count {
                let rt = self.info_reader.get_retention_time(func, i)?;
                cycle_index.push(CycleIndexEntry::new(func, i, rt, im_block_size, 0));
            }
        }

        cycle_index.sort_by(|a, b| a.time.total_cmp(&b.time));
        // let mut function_index: HashMap<usize, Vec<usize>> = HashMap::default();
        let mut spectrum_index = Vec::with_capacity(cycle_index.len());
        for (i, entry) in cycle_index.iter_mut().enumerate() {
            entry.index = i;
            // function_index.entry(entry.function).or_default().push(i);
            if entry.im_block_size > 0 {
                for j in 0..entry.im_block_size {
                    spectrum_index.push(SpectrumIndexEntry::new(
                        entry.function,
                        entry.block,
                        Some(j as u32),
                    ))
                }
            } else {
                spectrum_index.push(SpectrumIndexEntry::new(entry.function, entry.block, None))
            }
        }

        self.cycle_index = cycle_index;
        self.spectrum_index = spectrum_index;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn block_index(&self) -> &[CycleIndexEntry] {
        &self.cycle_index
    }

    pub fn index(&self) -> &[SpectrumIndexEntry] {
        &self.spectrum_index
    }

    pub fn len(&self) -> usize {
        self.spectrum_index.len()
    }

    pub fn get_spectrum(&mut self, index: usize) -> Option<Spectrum> {
        let entry = self.spectrum_index.get(index)?;

        let time = self
            .info_reader
            .get_retention_time(entry.function, entry.cycle)
            .ok()?;

        let ion_mode = self.info_reader.get_ion_mode(entry.function).ok()?;
        let is_continuum = self.info_reader.is_continuum(entry.function).ok()?;

        let spec = match entry.drift_index {
            Some(i) => {
                let (mzs, intens) = self
                    .scan_reader
                    .read_drift_scan(entry.function, entry.cycle, i as usize)
                    .ok()?;

                let drift_time = self
                    .info_reader
                    .get_drift_time(i as usize)
                    .ok();

                Spectrum::new(
                    mzs,
                    intens,
                    index,
                    time,
                    entry.clone(),
                    drift_time,
                    ion_mode,
                    is_continuum,
                )
            }
            None => {
                let (mzs, intens) = self
                    .scan_reader
                    .read_scan(entry.function, entry.cycle)
                    .ok()?;

                Spectrum::new(
                    mzs,
                    intens,
                    index,
                    time,
                    entry.clone(),
                    None,
                    ion_mode,
                    is_continuum,
                )
            }
        };

        Some(spec)
    }

    pub fn iter_spectra(&mut self) -> impl Iterator<Item = Spectrum> + '_ {
        (0..(self.len()))
            .into_iter()
            .flat_map(|i| self.get_spectrum(i))
    }

    pub fn tic_of(&mut self, which_function: usize) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut times = Vec::new();
        let mut intensities = Vec::new();
        self.chromatogram_reader
            .read_tic_into(which_function, &mut times, &mut intensities)?;

        Ok((times, intensities))
    }

    pub fn bpi_of(&mut self, which_function: usize) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut times = Vec::new();
        let mut intensities = Vec::new();
        self.chromatogram_reader
            .read_bpi_into(which_function, &mut times, &mut intensities)?;

        Ok((times, intensities))
    }

    pub fn tic(&mut self) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut chrom_slices: Vec<
            std::iter::Peekable<std::iter::Zip<std::vec::IntoIter<f32>, std::vec::IntoIter<f32>>>,
        > = Vec::new();

        for f in 0..self.info_reader.function_count()? {
            let mut times_of = Vec::new();
            let mut intensities_of = Vec::new();

            self.chromatogram_reader
                .read_tic_into(f, &mut times_of, &mut intensities_of)?;

            chrom_slices.push(
                times_of
                    .into_iter()
                    .zip(intensities_of.into_iter())
                    .peekable(),
            );
        }

        Ok(ChromatogramMerger::new(chrom_slices).merge())
    }

    pub fn bpi(&mut self) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut chrom_slices: Vec<
            std::iter::Peekable<std::iter::Zip<std::vec::IntoIter<f32>, std::vec::IntoIter<f32>>>,
        > = Vec::new();

        for f in 0..self.info_reader.function_count()? {
            let mut times_of = Vec::new();
            let mut intensities_of = Vec::new();

            self.chromatogram_reader
                .read_bpi_into(f, &mut times_of, &mut intensities_of)?;

            chrom_slices.push(
                times_of
                    .into_iter()
                    .zip(intensities_of.into_iter())
                    .peekable(),
            );
        }

        Ok(ChromatogramMerger::new(chrom_slices).merge())
    }
}

struct ChromatogramMerger {
    iters:
        Vec<std::iter::Peekable<std::iter::Zip<std::vec::IntoIter<f32>, std::vec::IntoIter<f32>>>>,
}

impl ChromatogramMerger {
    fn new(
        iters: Vec<
            std::iter::Peekable<std::iter::Zip<std::vec::IntoIter<f32>, std::vec::IntoIter<f32>>>,
        >,
    ) -> Self {
        Self { iters }
    }

    fn next_point(&mut self) -> Option<(f32, f32)> {
        self.iters
            .iter_mut()
            .map(|s| (s.peek().map(|(_, t)| *t).unwrap_or(f32::INFINITY), s))
            .reduce(|(cur_time, cur_it), (time, it)| {
                if time < cur_time {
                    (time, it)
                } else {
                    (cur_time, cur_it)
                }
            })
            .and_then(|(_, it)| it.next())
    }

    fn merge(mut self) -> (Vec<f32>, Vec<f32>) {
        let mut times = Vec::new();
        let mut intensities = Vec::new();

        while let Some((time, intens)) = self.next_point() {
            times.push(time);
            intensities.push(intens);
        }

        (times, intensities)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Spectrum {
    pub mz_array: Vec<f32>,
    pub intensity_array: Vec<f32>,
    pub index: usize,
    pub time: f64,
    pub identifier: SpectrumIndexEntry,
    pub drift_time: Option<f64>,
    pub ion_mode: MassLynxIonMode,
    pub is_continuum: bool,
}

impl Spectrum {
    pub fn new(
        mz_array: Vec<f32>,
        intensity_array: Vec<f32>,
        index: usize,
        time: f64,
        identifier: SpectrumIndexEntry,
        drift_time: Option<f64>,
        ion_mode: MassLynxIonMode,
        is_continuum: bool,
    ) -> Self {
        Self {
            mz_array,
            intensity_array,
            index,
            time,
            identifier,
            drift_time,
            ion_mode,
            is_continuum,
        }
    }

    pub fn function(&self) -> usize {
        self.identifier.function
    }
}
