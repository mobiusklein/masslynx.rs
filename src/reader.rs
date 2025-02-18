//! The higher-ish level API

use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    base::MassLynxChromatogramReader,
    constants::{
        AcquisitionParameter, LockMassParameter, MassLynxFunctionType, MassLynxHeaderItem, MassLynxIonMode, MassLynxScanItem
    },
    AsMassLynxSource, MassLynxError, MassLynxInfoReader, MassLynxLockMassProcessor,
    MassLynxParameters, MassLynxResult, MassLynxScanReader, MassLynxAnalogReader,
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

#[derive(Debug, Default, Clone, PartialEq)]
struct RawPaths {
    base_path: PathBuf,
    function_paths: HashMap<usize, PathBuf>,
    chromatogram_paths: HashMap<usize, PathBuf>,
}

impl RawPaths {
    fn function_has_cdt(&self, function: usize) -> bool {
        self.function_paths
            .get(&function)
            .map(|p| p.with_extension("cdt").exists())
            .unwrap_or_default()
    }

    fn from_path(base_path: PathBuf) -> io::Result<Self> {
        let mut this = Self {
            base_path,
            ..Self::default()
        };
        this.build_from_base()?;
        Ok(this)
    }

    fn build_from_base(&mut self) -> io::Result<()> {
        let root = self.base_path.as_path();
        let dirs = fs::read_dir(root)?;

        let func_regex = regex::Regex::new(r"_func0*(\d+).dat").unwrap();
        let chrom_regex = regex::Regex::new(r"_chro0*(\d+).dat").unwrap();

        for member in dirs.flatten() {
            if member.file_type()?.is_dir() {
                continue;
            }

            let name = member.file_name().to_string_lossy().to_lowercase();
            if name.starts_with("_func") && name.ends_with(".dat") {
                if let Some(pat) = func_regex.captures(&name) {
                    let func_num: usize = pat
                        .get(1)
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .unwrap_or_else(|e| {
                            panic!("Failed to parse function number from {name}: {e}")
                        })
                        .saturating_sub(1);
                    self.function_paths.insert(func_num, member.path());
                }
            }
            if name.starts_with("_chro") && name.ends_with(".dat") {
                if let Some(pat) = chrom_regex.captures(&name) {
                    let func_num: usize = pat
                        .get(1)
                        .unwrap()
                        .as_str()
                        .parse::<usize>()
                        .unwrap_or_else(|e| {
                            panic!("Failed to parse function number from {name}: {e}")
                        })
                        .saturating_sub(1);
                    self.chromatogram_paths.insert(func_num, member.path());
                }
            }
        }

        Ok(())
    }

    fn path(&self) -> &PathBuf {
        &self.base_path
    }
}

#[derive(Debug, Clone)]
pub struct ScanFunction {
    pub function: usize,
    pub ftype: MassLynxFunctionType,
    pub ms_level: u8,
    pub is_lockmass: bool,
    pub ion_mobility_block_size: usize,
    pub scan_count: usize,
    pub scan_items: Vec<MassLynxScanItem>,
}

impl ScanFunction {
    pub fn new(
        function: usize,
        ftype: MassLynxFunctionType,
        is_lockmass: bool,
        ion_mobility_block_size: usize,
        scan_count: usize,
        ms_level: u8,
        scan_items: Vec<MassLynxScanItem>,
    ) -> Self {
        Self {
            function,
            ftype,
            is_lockmass,
            ion_mobility_block_size,
            scan_count,
            ms_level,
            scan_items,
        }
    }

    pub fn is_sonar(&self) -> bool {
        self.scan_items.contains(&MassLynxScanItem::SONAR_ENABLED)
    }

    pub fn has_drift_time(&self) -> bool {
        self.ion_mobility_block_size > 0
    }
}

#[derive(Debug, Default)]
struct ScanReadingOptions {
    skip_lockmass: bool,
    load_signal: bool,
}

impl ScanReadingOptions {
    fn new(skip_lockmass: bool, load_signal: bool) -> Self {
        Self { skip_lockmass, load_signal }
    }

    fn skip_lockmass(&self) -> bool {
        self.skip_lockmass
    }

    fn set_skip_lockmass(&mut self, skip_lockmass: bool) {
        self.skip_lockmass = skip_lockmass;
    }

    fn set_load_signal(&mut self, load_signal: bool) {
        self.load_signal = load_signal;
    }

    fn load_signal(&self) -> bool {
        self.load_signal
    }
}

pub struct MassLynxReader {
    path: RawPaths,
    scan_reader: MassLynxScanReader,
    info_reader: MassLynxInfoReader,
    chromatogram_reader: MassLynxChromatogramReader,
    lockmass_processor: MassLynxLockMassProcessor,
    analog_reader: Option<MassLynxAnalogReader>,
    cycle_index: Vec<CycleIndexEntry>,
    spectrum_index: Vec<SpectrumIndexEntry>,
    scan_reading_options: ScanReadingOptions,
    functions: Vec<ScanFunction>,
}

impl MassLynxReader {
    pub fn from_path(path: &str) -> MassLynxResult<Self> {
        let info_reader = MassLynxInfoReader::from_path(&path)?;
        let scan_reader = MassLynxScanReader::from_source(&info_reader)?;
        let chromatogram_reader = MassLynxChromatogramReader::from_source(&info_reader)?;
        let analog_reader = MassLynxAnalogReader::from_source(&info_reader).ok();
        let mut lockmass_processor = MassLynxLockMassProcessor::new()?;
        lockmass_processor.set_raw_data_from_reader(&scan_reader)?;

        let path = RawPaths::from_path(PathBuf::from(path)).map_err(|e| MassLynxError {
            error_code: 9999,
            message: format!("Failed to build file name registry: {e}"),
        })?;

        let mut this = Self {
            path,
            info_reader,
            scan_reader,
            chromatogram_reader,
            analog_reader,
            lockmass_processor,
            cycle_index: Default::default(),
            spectrum_index: Default::default(),
            scan_reading_options: ScanReadingOptions::new(true, true),
            functions: Vec::new(),
        };

        this.functions = this.describe_functions()?;
        this.build_index()?;
        Ok(this)
    }

    /// Describe the scan functions found in this run
    pub fn functions(&self) -> &[ScanFunction] {
        &self.functions
    }

    fn describe_functions(&mut self) -> MassLynxResult<Vec<ScanFunction>> {
        let lockmass_fn = self.get_lock_mass_function();
        let n_funcs = self.info_reader.function_count()?;

        let mut functions = Vec::new();
        for fnum in 0..n_funcs {
            let ftype = self.info_reader.get_function_type(fnum)?;

            let scan_count = self.info_reader.scan_count_for_function(fnum)?;
            let im_block_size = if self.path.function_has_cdt(fnum) {
                self.info_reader
                    .get_drift_scan_count(fnum)
                    .ok()
                    .unwrap_or_default()
            } else {
                0
            };

            let ms_level = self.translate_function_type_to_ms_level(fnum)?;

            let scan_items = self.info_reader.get_scan_items(fnum)?.iter_keys().collect();

            let descr = ScanFunction::new(
                fnum,
                ftype,
                Some(fnum) == lockmass_fn,
                im_block_size,
                scan_count,
                ms_level,
                scan_items,
            );
            functions.push(descr);
        }

        Ok(functions)
    }

    /// Get the index of the lock mass function
    pub fn get_lock_mass_function(&self) -> Option<usize> {
        self.info_reader
            .get_lock_mass_function()
            .ok()
            .map(|(_, func)| func)
    }

    /// Check if the run is lock mass corrected
    pub fn is_lock_mass_corrected(&mut self) -> bool {
        self.info_reader
            .is_lock_mass_corrected()
            .unwrap_or_default()
    }

    /// Manually set the lock mass target
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

        for func in self.functions.iter() {
            if func.ms_level == 0 {
                continue;
            }

            for i in 0..func.scan_count {
                let rt = self.info_reader.get_retention_time(func.function, i)?;
                cycle_index.push(CycleIndexEntry::new(
                    func.function,
                    i,
                    rt,
                    func.ion_mobility_block_size,
                    0,
                ));
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

    /// Get the base path of the RAW directory
    pub fn path(&self) -> &Path {
        &self.path.path()
    }

    /// Get an index over the function cycles
    pub fn cycle_index(&self) -> &[CycleIndexEntry] {
        &self.cycle_index
    }

    /// Get an index over the spectra
    pub fn index(&self) -> &[SpectrumIndexEntry] {
        &self.spectrum_index
    }

    /// Get the number of raw spectra in the run
    pub fn len(&self) -> usize {
        self.spectrum_index.len()
    }

    pub fn read_scan_items(
        &mut self,
        which_function: usize,
        scan: usize,
    ) -> MassLynxResult<Vec<(MassLynxScanItem, String)>> {
        if let Some(f) = self.functions.get(which_function) {
            let params_values = self.info_reader.get_scan_item_values_for_scan(
                which_function,
                scan,
                &f.scan_items,
            )?;
            let items: Vec<_> = params_values.iter::<MassLynxScanItem>().collect();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn get_spectrum(&mut self, index: usize) -> Option<Spectrum> {
        let entry = *self.spectrum_index.get(index)?;

        let time = self
            .info_reader
            .get_retention_time(entry.function, entry.cycle)
            .ok()?;

        let ion_mode = self.info_reader.get_ion_mode(entry.function).ok()?;
        let is_continuum = self.info_reader.is_continuum(entry.function).ok()?;

        let items = self.read_scan_items(entry.function, entry.cycle).ok()?;

        let spec = match entry.drift_index {
            Some(i) => {
                let (mzs, intens) = if self.scan_reading_options.load_signal { self
                    .scan_reader
                    .read_drift_scan(entry.function, entry.cycle, i as usize)
                    .ok()?
                } else {
                    (Vec::new(), Vec::new())
                };

                let drift_time = self.info_reader.get_drift_time(i as usize).ok();

                Spectrum::new(
                    mzs,
                    intens,
                    index,
                    time,
                    entry,
                    drift_time,
                    ion_mode,
                    is_continuum,
                    items,
                )
            }
            None => {
                let (mzs, intens) = if self.scan_reading_options.load_signal { self
                    .scan_reader
                    .read_scan(entry.function, entry.cycle)
                    .ok()?
                } else {
                    Default::default()
                };

                Spectrum::new(
                    mzs,
                    intens,
                    index,
                    time,
                    entry,
                    None,
                    ion_mode,
                    is_continuum,
                    items,
                )
            }
        };

        Some(spec)
    }

    pub fn iter_spectra(&mut self) -> impl Iterator<Item = Spectrum> + '_ {
        (0..(self.len())).flat_map(|i| self.get_spectrum(i))
    }

    pub fn get_cycle(&mut self, index: usize) -> Option<Cycle> {
        let entry = *self.cycle_index.get(index)?;

        if self.scan_reading_options.skip_lockmass && self.functions[entry.function].is_lockmass {
            return None;
        }

        let time = self
            .info_reader
            .get_retention_time(entry.function, entry.block)
            .ok()?;

        let ion_mode = self.info_reader.get_ion_mode(entry.function).ok()?;
        let is_continuum = self.info_reader.is_continuum(entry.function).ok()?;

        let scans = if self.scan_reading_options.load_signal {
            let mut scans = Vec::with_capacity(entry.im_block_size);
            for i in 0..entry.im_block_size {
                let (mzs, intensities) = self
                    .scan_reader
                    .read_drift_scan(entry.function, entry.block, i)
                    .ok()?;
                let drift_time = self.info_reader.get_drift_time(i).ok()?;
                scans.push(DriftScan::new(drift_time, mzs, intensities));
            }
            scans
        } else {
            Vec::new()
        };

        let items = self.read_scan_items(entry.function, entry.block).ok()?;

        Some(Cycle::new(
            scans,
            index,
            entry,
            time,
            ion_mode,
            is_continuum,
            items,
        ))
    }

    pub fn iter_cycles(&mut self) -> impl Iterator<Item = Cycle> + '_ {
        (0..(self.cycle_index.len())).flat_map(|i| self.get_cycle(i))
    }

    pub fn get_signal_loading(&self) -> bool {
        self.scan_reading_options.load_signal()
    }

    pub fn set_signal_loading(&mut self, load_signal: bool) {
        self.scan_reading_options.set_load_signal(load_signal)
    }

    pub fn get_lockmass_skipping(&self) -> bool {
        self.scan_reading_options.skip_lockmass()
    }

    pub fn set_lockmass_skipping(&mut self, skip_lockmass: bool) {
        self.scan_reading_options.set_skip_lockmass(skip_lockmass)
    }
}


/// Read chromatograms and mobilograms
impl MassLynxReader {
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

    pub fn read_xic(
        &mut self,
        which_function: usize,
        mass: f32,
        mass_window: f32,
        daughters: bool,
    ) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut time_array = Vec::new();
        let mut intensity_array = Vec::new();

        self.chromatogram_reader.read_mass_chromatogram_into(
            which_function,
            mass,
            &mut time_array,
            &mut intensity_array,
            mass_window,
            daughters,
        )?;

        Ok((time_array, intensity_array))
    }

    pub fn read_xics(
        &mut self,
        which_function: usize,
        masses: &[f32],
        mass_window: f32,
        daughters: bool,
    ) -> MassLynxResult<Vec<(Arc<Vec<f32>>, Vec<f32>)>> {
        let mut time_array = Vec::new();
        let mut intensity_arrays: Vec<_> = (0..(masses.len())).map(|_| Vec::new()).collect();

        self.chromatogram_reader.read_mass_chromatograms_into(
            which_function,
            masses,
            &mut time_array,
            &mut intensity_arrays,
            mass_window,
            daughters,
        )?;

        let time_array = Arc::new(time_array);
        let mut xics = Vec::new();
        for ints in intensity_arrays {
            xics.push((Arc::clone(&time_array), ints));
        }

        Ok(xics)
    }

    pub fn read_mobilogram(
        &mut self,
        which_function: usize,
        start_scan: usize,
        end_scan: usize,
        start_mass: f32,
        end_mass: f32,
    ) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut drift_bins = Vec::new();
        let mut intensity_array = Vec::new();
        self.chromatogram_reader.read_mobilogram_into(
            which_function,
            start_scan,
            end_scan,
            start_mass,
            end_mass,
            &mut drift_bins,
            &mut intensity_array,
        )?;
        let drift_times: MassLynxResult<Vec<f32>> = drift_bins
            .into_iter()
            .map(|i| {
                self.info_reader
                    .get_drift_time(i as usize)
                    .map(|f| f as f32)
            })
            .collect();
        Ok((drift_times?, intensity_array))
    }

    pub fn iter_analogs(&mut self) -> impl Iterator<Item=Trace> + '_ {
        let num_analog_traces = self.analog_reader.as_mut().and_then(|ar| {
            ar.channel_count().ok()
        }).unwrap_or_default();

        (0..num_analog_traces).flat_map(|i| -> MassLynxResult<Trace> {
            let reader = self.analog_reader.as_mut().unwrap();
            let (time, intensity) = reader.read_channel(i)?;
            let name = reader.channel_description(i)?;
            let unit = reader.channel_units(i)?;
            Ok(Trace::new(name, unit, time, intensity))
        })
    }

    pub fn get_analog_trace(&mut self, index: usize) -> Option<Trace> {
        let num_analog_traces = self.analog_reader.as_mut().and_then(|ar| {
            ar.channel_count().ok()
        }).unwrap_or_default();
        if index >= num_analog_traces {
            return None
        }
        self.analog_reader.as_mut().and_then(|reader| {
            let (time, intensity) = reader.read_channel(index).ok()?;
            let name = reader.channel_description(index).ok()?;
            let unit = reader.channel_units(index).ok()?;
            Some(Trace::new(name, unit, time, intensity))
        })
    }
}


/// General metadata reading
impl MassLynxReader {
    pub fn read_headers_from_file(&self) -> io::Result<HashMap<String, String>> {
        let mut headers_path = self.path().join("_header.txt");
        let mut headers: HashMap<String, String> = HashMap::new();

        if !headers_path.exists() {
            headers_path = self.path().join("_HEADER.TXT");
            if !headers_path.exists() {
                return Ok(headers);
            }
        }

        let handle = io::BufReader::new(fs::File::open(headers_path)?);

        for line in handle.lines().flatten() {
            if !line.starts_with("$$ ") {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers
                    .entry(key[3..].trim_ascii().to_string())
                    .insert_entry(value.trim().to_string());
            }
        }

        Ok(headers)
    }

    pub fn header_items(&self) -> MassLynxResult<Vec<(MassLynxHeaderItem, String)>> {
        let items: Vec<_> = MassLynxHeaderItem::iter().collect();
        let items = self.info_reader.get_header_items(&items)?;
        let header_items: Vec<(MassLynxHeaderItem, String)> =
            items.iter().filter(|(_, v)| !v.is_empty()).collect();
        Ok(header_items)
    }

    pub fn acquisition_information(&mut self) -> MassLynxResult<HashMap<AcquisitionParameter, String>> {
        Ok(self.info_reader.get_acquisition_info()?.to_hashmap())
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
    pub items: Vec<(MassLynxScanItem, String)>,
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
        items: Vec<(MassLynxScanItem, String)>,
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
            items,
        }
    }

    pub fn function(&self) -> usize {
        self.identifier.function
    }

    pub fn native_id(&self) -> String {
        self.identifier.native_id()
    }
}

#[derive(Debug, Default, Clone)]
pub struct DriftScan {
    pub drift_time: f64,
    pub mz_array: Vec<f32>,
    pub intensity_array: Vec<f32>,
}

impl DriftScan {
    pub fn new(drift_time: f64, mz_array: Vec<f32>, intensity_array: Vec<f32>) -> Self {
        Self {
            drift_time,
            mz_array,
            intensity_array,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Cycle {
    pub signal: Vec<DriftScan>,
    pub index: usize,
    pub identifier: CycleIndexEntry,
    pub time: f64,
    pub ion_mode: MassLynxIonMode,
    pub is_continuum: bool,
    pub items: Vec<(MassLynxScanItem, String)>,
}

impl Cycle {
    pub fn new(
        signal: Vec<DriftScan>,
        index: usize,
        identifier: CycleIndexEntry,
        time: f64,
        ion_mode: MassLynxIonMode,
        is_continuum: bool,
        items: Vec<(MassLynxScanItem, String)>,
    ) -> Self {
        Self {
            signal,
            index,
            identifier,
            time,
            ion_mode,
            is_continuum,
            items,
        }
    }

    pub fn function(&self) -> usize {
        self.identifier.function
    }

    pub fn native_id(&self) -> String {
        self.identifier.native_id()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Trace {
    pub name: String,
    pub unit: String,
    pub time: Vec<f32>,
    pub intensity: Vec<f32>,
}

impl Trace {
    pub fn new(name: String, unit: String, time: Vec<f32>, intensity: Vec<f32>) -> Self {
        Self { name, unit, time, intensity }
    }
}
