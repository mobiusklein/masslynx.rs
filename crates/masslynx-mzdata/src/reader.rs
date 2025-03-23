use std::{fs, io, marker::PhantomData, path::Path};

use mzdata::{
    curie, delegate_impl_metadata_trait,
    io::{DetailLevel, OffsetIndex},
    meta::{
        FileDescription, FileMetadataConfig, InstrumentConfiguration,
        MassSpectrometerFileFormatTerm, MassSpectrometryRun, NativeSpectrumIdentifierFormatTerm,
        Sample, Software, SoftwareTerm, SourceFile,
    },
    mzpeaks::{IonMobility, Mass, MZ},
    params::{ControlledVocabulary::MS, Unit},
    prelude::*,
    spectrum::{
        bindata::{ArrayRetrievalError, BinaryArrayMap3D},
        Acquisition, Activation, ArrayType, BinaryArrayMap, BinaryDataArrayType, Chromatogram,
        ChromatogramDescription, ChromatogramType, DataArray, IonMobilityFrameDescription,
        MultiLayerIonMobilityFrame, MultiLayerSpectrum, Precursor, ScanPolarity, ScanWindow,
        SelectedIon, SignalContinuity, SpectrumDescription,
    },
    Param,
};

#[allow(unused)]
use mzdata::io::checksum_file;

use masslynx::{
    reader::{MassLynxReader, ScanFunction, Spectrum, Trace},
    MassLynxHeaderItem,
};

fn build_file_description(reader: &MassLynxReader) -> io::Result<FileDescription> {
    let root = reader.path();
    let dirs = fs::read_dir(root)?;

    let mut desc = FileDescription::default();
    for member in dirs.flatten() {
        if member.file_type()?.is_dir() {
            continue;
        }

        let mut sf = SourceFile::from_path(&member.path())?;
        sf.file_format = Some(MassSpectrometerFileFormatTerm::WatersRaw.into());
        sf.id_format = Some(NativeSpectrumIdentifierFormatTerm::WatersNativeIDFormat.into());
        #[cfg(not(debug_assertions))]
        sf.add_param(ControlledVocabulary::MS.param_val(
            1000569u32,
            "SHA-1",
            checksum_file(&member.path())?,
        ));
        sf.id = member
            .path()
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap();

        desc.source_files.push(sf);
    }

    let mut has_ms1 = false;
    let mut has_msn = false;

    for func in reader.functions() {
        if func.ms_level == 1 {
            has_ms1 = true;
        }
        if func.ms_level > 1 {
            has_msn = true;
        }
    }

    if has_ms1 {
        desc.add_param(
            Param::builder()
                .name("MS1 spectrum")
                .curie(curie!(MS:1000579))
                .build(),
        )
    }

    if has_msn {
        desc.add_param(
            Param::builder()
                .name("MSn spectrum")
                .curie(curie!(MS:1000580))
                .build(),
        )
    }

    Ok(desc)
}

pub struct MassLynxCycleReaderType<
    C: FeatureLike<MZ, IonMobility>,
    D: FeatureLike<Mass, IonMobility> + KnownCharge,
> {
    reader: MassLynxReader,
    index: usize,
    frame_index: OffsetIndex,
    detail_level: DetailLevel,
    metadata: FileMetadataConfig,
    _d: PhantomData<(C, D)>,
}

impl<C: FeatureLike<MZ, IonMobility>, D: FeatureLike<Mass, IonMobility> + KnownCharge>
    ChromatogramSource for MassLynxCycleReaderType<C, D>
{
    fn get_chromatogram_by_id(&mut self, id: &str) -> Option<Chromatogram> {
        match id {
            "TIC" => self.get_chromatogram(0),
            "BPC" => self.get_chromatogram(1),
            _ => {
                let trace = self.reader.iter_analogs().find(|t| t.name == id)?;
                Some(self.trace_to_chromatogram(&trace))
            }
        }
    }

    fn get_chromatogram_by_index(&mut self, index: usize) -> Option<Chromatogram> {
        self.get_chromatogram(index)
    }
}

impl<C: FeatureLike<MZ, IonMobility>, D: FeatureLike<Mass, IonMobility> + KnownCharge>
    MSDataFileMetadata for MassLynxCycleReaderType<C, D>
{
    delegate_impl_metadata_trait!(metadata);
}

impl<C: FeatureLike<MZ, IonMobility>, D: FeatureLike<Mass, IonMobility> + KnownCharge>
    RandomAccessIonMobilityFrameIterator<C, D, MultiLayerIonMobilityFrame<C, D>>
    for MassLynxCycleReaderType<C, D>
{
    fn start_from_id(
        &mut self,
        id: &str,
    ) -> Result<&mut Self, mzdata::io::IonMobilityFrameAccessError> {
        match self.frame_index.get(id) {
            Some(i) => {
                self.index = i as usize;
                Ok(self)
            }
            None => Err(mzdata::io::IonMobilityFrameAccessError::FrameIdNotFound(
                id.to_string(),
            )),
        }
    }

    fn start_from_index(
        &mut self,
        index: usize,
    ) -> Result<&mut Self, mzdata::io::IonMobilityFrameAccessError> {
        if index > self.len() {
            Err(mzdata::io::IonMobilityFrameAccessError::FrameIndexNotFound(
                index,
            ))
        } else {
            self.index = index;
            Ok(self)
        }
    }

    fn start_from_time(
        &mut self,
        time: f64,
    ) -> Result<&mut Self, mzdata::io::IonMobilityFrameAccessError> {
        match self._offset_of_time(time) {
            Some(i) => {
                self.index = i as usize;
                Ok(self)
            }
            None => Err(mzdata::io::IonMobilityFrameAccessError::FrameNotFound),
        }
    }
}

impl<C: FeatureLike<MZ, IonMobility>, D: FeatureLike<Mass, IonMobility> + KnownCharge>
    IonMobilityFrameSource<C, D, MultiLayerIonMobilityFrame<C, D>>
    for MassLynxCycleReaderType<C, D>
{
    fn reset(&mut self) {
        self.index = 0;
    }

    fn detail_level(&self) -> &mzdata::io::DetailLevel {
        &self.detail_level
    }

    fn set_detail_level(&mut self, detail_level: mzdata::io::DetailLevel) {
        self.detail_level = detail_level;
        match self.detail_level {
            DetailLevel::Full => {
                self.reader.set_signal_loading(true);
            }
            DetailLevel::Lazy => {
                self.detail_level = DetailLevel::Full;
                self.reader.set_signal_loading(true);
            }
            DetailLevel::MetadataOnly => {
                self.reader.set_signal_loading(false);
            }
        }
    }

    fn get_frame_by_id(&mut self, id: &str) -> Option<MultiLayerIonMobilityFrame<C, D>> {
        self.frame_index
            .get(id)
            .and_then(|i| self.get_frame(i as usize))
    }

    fn get_frame_by_index(&mut self, index: usize) -> Option<MultiLayerIonMobilityFrame<C, D>> {
        self.get_frame(index)
    }

    fn get_index(&self) -> &OffsetIndex {
        &self.frame_index
    }

    fn set_index(&mut self, index: OffsetIndex) {
        self.frame_index = index;
    }
}

impl<C: FeatureLike<MZ, IonMobility>, D: FeatureLike<Mass, IonMobility> + KnownCharge> Iterator
    for MassLynxCycleReaderType<C, D>
{
    type Item = MultiLayerIonMobilityFrame<C, D>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.get_frame(self.index);
        self.index += 1;
        frame
    }
}

impl<C: FeatureLike<MZ, IonMobility>, D: FeatureLike<Mass, IonMobility> + KnownCharge>
    MassLynxCycleReaderType<C, D>
{
    pub fn open_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let reader = MassLynxReader::from_path(path).map_err(|e| {
            io::Error::new(
                match e.error_code {
                    5 => {
                        if path.exists() {
                            io::ErrorKind::Other
                        } else {
                            io::ErrorKind::NotFound
                        }
                    }
                    _ => io::ErrorKind::Other,
                },
                e,
            )
        })?;
        let mut frame_index = OffsetIndex::new("spectrum".into());

        reader.cycle_index().iter().enumerate().for_each(|(i, c)| {
            frame_index.insert(c.native_id(), i as u64);
        });

        let header = reader
            .header_items()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut metadata = FileMetadataConfig::default();
        let mut instr = InstrumentConfiguration::default();

        let mut run_info = MassSpectrometryRun::default();
        let mut sample = Sample::default();

        for (k, v) in header {
            match k {
                MassLynxHeaderItem::VERSION => {}
                MassLynxHeaderItem::ACQUIRED_NAME => {
                    run_info.id = Some(v.clone());
                    sample.name = Some(v.clone());
                }
                MassLynxHeaderItem::ACQUIRED_DATE => {
                    let dt = run_info.start_time.get_or_insert_default();
                    *dt = v.parse().expect("Failed to parse date");
                }
                MassLynxHeaderItem::ACQUIRED_TIME => {
                    let dt = run_info.start_time.get_or_insert_default();
                    *dt = dt
                        .with_time(v.parse().expect("Failed to parse time"))
                        .unwrap();
                }
                MassLynxHeaderItem::JOB_CODE => {}
                MassLynxHeaderItem::TASK_CODE => {}
                MassLynxHeaderItem::USER_NAME => {}
                MassLynxHeaderItem::INSTRUMENT => {
                    instr.add_param(
                        Param::builder()
                            .accession(1000529)
                            .controlled_vocabulary(MS)
                            .name("instrument serial number")
                            .value(v)
                            .build(),
                    );
                }
                MassLynxHeaderItem::CONDITIONS => {}
                MassLynxHeaderItem::LAB_NAME => {}
                MassLynxHeaderItem::SAMPLE_DESCRIPTION => {
                    sample.add_param(Param::new_key_value("sample description", v));
                }
                MassLynxHeaderItem::SOLVENT_DELAY => {}
                MassLynxHeaderItem::SUBMITTER => {}
                MassLynxHeaderItem::SAMPLE_ID => {
                    sample.add_param(
                        Param::builder()
                            .name("sample number")
                            .accession(1000001)
                            .controlled_vocabulary(MS)
                            .value(v)
                            .build(),
                    );
                }
                MassLynxHeaderItem::BOTTLE_NUMBER => {}
                MassLynxHeaderItem::ANALOG_CH1_OFFSET => {}
                MassLynxHeaderItem::ANALOG_CH2_OFFSET => {}
                MassLynxHeaderItem::ANALOG_CH3_OFFSET => {}
                MassLynxHeaderItem::ANALOG_CH4_OFFSET => {}
                MassLynxHeaderItem::CAL_MS1_STATIC => {}
                MassLynxHeaderItem::CAL_MS2_STATIC => {}
                MassLynxHeaderItem::CAL_MS1_STATIC_PARAMS => {}
                MassLynxHeaderItem::CAL_MS1_DYNAMIC_PARAMS => {}
                MassLynxHeaderItem::CAL_MS2_STATIC_PARAMS => {}
                MassLynxHeaderItem::CAL_MS2_DYNAMIC_PARAMS => {}
                MassLynxHeaderItem::CAL_MS1_FAST_PARAMS => {}
                MassLynxHeaderItem::CAL_MS2_FAST_PARAMS => {}
                MassLynxHeaderItem::CAL_TIME => {}
                MassLynxHeaderItem::CAL_DATE => {}
                MassLynxHeaderItem::CAL_TEMPERATURE => {}
                MassLynxHeaderItem::INLET_METHOD => {}
                MassLynxHeaderItem::SPARE1 => {}
                MassLynxHeaderItem::SPARE2 => {}
                MassLynxHeaderItem::SPARE3 => {}
                MassLynxHeaderItem::SPARE4 => {}
                MassLynxHeaderItem::SPARE5 => {}
            }
        }

        instr.id = 0;
        metadata
            .instrument_configurations_mut()
            .insert(instr.id, instr);
        metadata.samples_mut().push(sample);
        *metadata.file_description_mut() = build_file_description(&reader)?;

        let sw = Software::new(
            "masslynx".into(),
            masslynx::get_mass_lynx_version().unwrap(),
            vec![SoftwareTerm::MassLynx.into()],
        );

        metadata.softwares_mut().push(sw);

        Ok(Self {
            reader,
            index: 0,
            frame_index,
            metadata,
            detail_level: DetailLevel::Full,
            _d: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.reader.cycle_index().len()
    }

    pub fn is_empty(&self) -> bool {
        self.reader.cycle_index().is_empty()
    }

    pub(crate) fn get_tic(&mut self) -> Option<Chromatogram> {
        let (times, intensities) = self.reader.tic().ok()?;
        let mut time_array = DataArray::from_name_type_size(
            &ArrayType::TimeArray,
            BinaryDataArrayType::Float64,
            times.len() * BinaryDataArrayType::Float64.size_of(),
        );

        for value in times.iter().map(|v| *v as f64) {
            time_array.push(value).unwrap();
        }

        let mut intensity_array = DataArray::from_name_type_size(
            &ArrayType::IntensityArray,
            BinaryDataArrayType::Float32,
            BinaryDataArrayType::Float32.size_of() * intensities.len(),
        );

        intensity_array.extend(&intensities).unwrap();

        let mut arrays = BinaryArrayMap::new();
        arrays.add(time_array);
        arrays.add(intensity_array);

        let mut desc = ChromatogramDescription::default();
        desc.id = "TIC".into();
        desc.chromatogram_type = ChromatogramType::TotalIonCurrentChromatogram;

        Some(Chromatogram::new(desc, arrays))
    }

    pub(crate) fn get_bpc(&mut self) -> Option<Chromatogram> {
        let (times, intensities) = self.reader.bpi().ok()?;
        let mut time_array = DataArray::from_name_type_size(
            &ArrayType::TimeArray,
            BinaryDataArrayType::Float64,
            times.len() * BinaryDataArrayType::Float64.size_of(),
        );

        for value in times.iter().map(|v| *v as f64) {
            time_array.push(value).unwrap();
        }

        let mut intensity_array = DataArray::from_name_type_size(
            &ArrayType::IntensityArray,
            BinaryDataArrayType::Float32,
            BinaryDataArrayType::Float32.size_of() * intensities.len(),
        );

        intensity_array.extend(&intensities).unwrap();

        let mut arrays = BinaryArrayMap::new();
        arrays.add(time_array);
        arrays.add(intensity_array);

        let mut desc = ChromatogramDescription::default();
        desc.id = "BPC".into();
        desc.chromatogram_type = ChromatogramType::BasePeakChromatogram;

        Some(Chromatogram::new(desc, arrays))
    }

    pub(crate) fn trace_to_chromatogram(&self, trace: &Trace) -> Chromatogram {
        let mut desc = ChromatogramDescription::default();

        let unit_str = if trace.unit.as_bytes().starts_with(&[239, 191, 189]) {
            let trimmed = String::from_utf8_lossy(&trace.unit.as_bytes()[3..]);
            let trimmed = trimmed.trim().to_string();
            trimmed
        } else {
            trace.unit.as_str().to_string()
        };

        let mut time_array = DataArray::from_name_type_size(
            &ArrayType::TimeArray,
            BinaryDataArrayType::Float64,
            trace.time.len() * BinaryDataArrayType::Float64.size_of(),
        );

        for value in trace.time.iter().map(|v| *v as f64) {
            time_array.push(value).unwrap();
        }

        let mut intensity_array = DataArray::from_name_type_size(
            &ArrayType::IntensityArray,
            BinaryDataArrayType::Float32,
            BinaryDataArrayType::Float32.size_of() * trace.intensity.len(),
        );

        intensity_array.extend(&trace.intensity).unwrap();

        match unit_str.as_str() {
            "C" => {
                desc.chromatogram_type = ChromatogramType::TemperatureChromatogram;
                intensity_array.name = ArrayType::TemperatureArray;
                // TODO: update units
            }
            "L/min" => {
                desc.chromatogram_type = ChromatogramType::FlowRateChromatogram;
                intensity_array.name = ArrayType::FlowRateArray;
                // TODO: update units
            }
            "psi" => {
                desc.chromatogram_type = ChromatogramType::PressureChromatogram;
                intensity_array.name = ArrayType::PressureArray;
                // TODO: update units
            }
            "%" => {

                // intensity_array.unit = Unit::Percent
            }
            _ => {
                desc.chromatogram_type = ChromatogramType::Unknown;
                intensity_array.add_param(Param::new_key_value("units", unit_str));
            } // _ => Unit::Unknown
        }

        let mut arrays = BinaryArrayMap::new();
        arrays.add(time_array);
        arrays.add(intensity_array);

        Chromatogram::new(desc, arrays)
    }

    pub(crate) fn get_chromatogram(&mut self, index: usize) -> Option<Chromatogram> {
        if index == 0 {
            self.get_tic()
        } else if index == 1 {
            self.get_bpc()
        } else {
            let trace: Trace = self.reader.get_analog_trace(index - 2)?;
            Some(self.trace_to_chromatogram(&trace))
        }
    }

    pub(crate) fn get_frame(&mut self, index: usize) -> Option<MultiLayerIonMobilityFrame<C, D>> {
        let cycle = self.reader.get_cycle(index)?;
        let func: &ScanFunction = &self.reader.functions()[cycle.function()];
        let id = cycle.native_id();
        let ms_level = func.ms_level;

        let polarity = match cycle.ion_mode {
            masslynx::MassLynxIonMode::EI_POS
            | masslynx::MassLynxIonMode::CI_POS
            | masslynx::MassLynxIonMode::FB_POS
            | masslynx::MassLynxIonMode::TS_POS
            | masslynx::MassLynxIonMode::ES_POS
            | masslynx::MassLynxIonMode::AI_POS
            | masslynx::MassLynxIonMode::LD_POS => ScanPolarity::Positive,

            masslynx::MassLynxIonMode::EI_NEG
            | masslynx::MassLynxIonMode::CI_NEG
            | masslynx::MassLynxIonMode::FB_NEG
            | masslynx::MassLynxIonMode::TS_NEG
            | masslynx::MassLynxIonMode::ES_NEG
            | masslynx::MassLynxIonMode::AI_NEG
            | masslynx::MassLynxIonMode::LD_NEG => ScanPolarity::Negative,
            masslynx::MassLynxIonMode::UNINITIALISED => ScanPolarity::Unknown,
        };

        let signal_continuity = if cycle.is_continuum {
            SignalContinuity::Profile
        } else {
            SignalContinuity::Centroid
        };

        let ion_mobility_unit = Unit::Millisecond;

        let mut params = Vec::new();

        let start_drift = cycle
            .signal
            .first()
            .map(|s| s.drift_time)
            .unwrap_or_default();
        let end_drift = cycle
            .signal
            .last()
            .map(|s| s.drift_time)
            .unwrap_or_default();

        params.add_param(
            Param::new_key_value("ion mobility lower limit", start_drift)
                .with_unit_t(&Unit::Millisecond),
        );
        params.add_param(
            Param::new_key_value("ion mobility upper limit", end_drift)
                .with_unit_t(&Unit::Millisecond),
        );

        let mut acquisition = Acquisition::default();
        let scan = acquisition.first_scan_mut().unwrap();
        scan.start_time = cycle.time;
        scan.add_param(
            Param::builder()
                .accession(1000616)
                .controlled_vocabulary(MS)
                .name("preset scan configuration")
                .value(func.function + 1)
                .build(),
        );
        if let Some((lo, hi)) = func.scan_range {
            scan.scan_windows
                .push(ScanWindow::new(lo as f32, hi as f32))
        }

        let mut precursor: Option<Precursor> = None;
        let mut ion: Option<SelectedIon> = None;
        let mut activation: Option<Activation> = None;

        for (key, val) in cycle.items.iter() {
            match key {
                masslynx::MassLynxScanItem::LINEAR_DETECTOR_VOLTAGE => {}
                masslynx::MassLynxScanItem::LINEAR_SENSITIVITY => {}
                masslynx::MassLynxScanItem::REFLECTRON_LENS_VOLTAGE => {}
                masslynx::MassLynxScanItem::REFLECTRON_DETECTOR_VOLTAGE => {}
                masslynx::MassLynxScanItem::REFLECTRON_SENSITIVITY => {}
                masslynx::MassLynxScanItem::LASER_REPETITION_RATE => {}
                masslynx::MassLynxScanItem::COURSE_LASER_CONTROL => {}
                masslynx::MassLynxScanItem::FINE_LASER_CONTROL => {}
                masslynx::MassLynxScanItem::LASERAIM_XPOS => {}
                masslynx::MassLynxScanItem::LASERAIM_YPOS => {}
                masslynx::MassLynxScanItem::NUM_SHOTS_SUMMED => {}
                masslynx::MassLynxScanItem::NUM_SHOTS_PERFORMED => {}
                masslynx::MassLynxScanItem::SEGMENT_NUMBER => {}
                masslynx::MassLynxScanItem::LCMP_TFM_WELL => {}
                masslynx::MassLynxScanItem::SEGMENT_TYPE => {}
                masslynx::MassLynxScanItem::SOURCE_REGION1 => {}
                masslynx::MassLynxScanItem::SOURCE_REGION2 => {}
                masslynx::MassLynxScanItem::REFLECTRON_FIELD_LENGTH => {}
                masslynx::MassLynxScanItem::REFLECTRON_LENGTH => {}
                masslynx::MassLynxScanItem::REFLECTRON_VOLT => {}
                masslynx::MassLynxScanItem::SAMPLE_PLATE_VOLT => {}
                masslynx::MassLynxScanItem::REFLECTRON_FIELD_LENGTH_ALT => {}
                masslynx::MassLynxScanItem::REFLECTRON_LENGTH_ALT => {}
                masslynx::MassLynxScanItem::PSD_STEP_MAJOR => {}
                masslynx::MassLynxScanItem::PSD_STEP_MINOR => {}
                masslynx::MassLynxScanItem::PSD_FACTOR_1 => {}
                masslynx::MassLynxScanItem::NEEDLE => {}
                masslynx::MassLynxScanItem::COUNTER_ELECTRODE_VOLTAGE => {}
                masslynx::MassLynxScanItem::SAMPLING_CONE_VOLTAGE => {}
                masslynx::MassLynxScanItem::SKIMMER_LENS => {}
                masslynx::MassLynxScanItem::SKIMMER => {}
                masslynx::MassLynxScanItem::PROBE_TEMPERATURE => {}
                masslynx::MassLynxScanItem::SOURCE_TEMPERATURE => {}
                masslynx::MassLynxScanItem::RF_VOLTAGE => {}
                masslynx::MassLynxScanItem::SOURCE_APERTURE => {}
                masslynx::MassLynxScanItem::SOURCE_CODE => {}
                masslynx::MassLynxScanItem::LM_RESOLUTION => {}
                masslynx::MassLynxScanItem::HM_RESOLUTION => {}
                masslynx::MassLynxScanItem::COLLISION_ENERGY => {
                    activation.get_or_insert_default().energy = val
                        .parse()
                        .unwrap_or_else(|e| panic!("Failed to parse COLLISION_ENERGY: {e}"))
                }
                masslynx::MassLynxScanItem::ION_ENERGY => {}
                masslynx::MassLynxScanItem::MULTIPLIER1 => {}
                masslynx::MassLynxScanItem::MULTIPLIER2 => {}
                masslynx::MassLynxScanItem::TRANSPORTDC => {}
                masslynx::MassLynxScanItem::TOF_APERTURE => {}
                masslynx::MassLynxScanItem::ACC_VOLTAGE => {}
                masslynx::MassLynxScanItem::STEERING => {}
                masslynx::MassLynxScanItem::FOCUS => {}
                masslynx::MassLynxScanItem::ENTRANCE => {}
                masslynx::MassLynxScanItem::GUARD => {}
                masslynx::MassLynxScanItem::TOF => {}
                masslynx::MassLynxScanItem::REFLECTRON => {}
                masslynx::MassLynxScanItem::COLLISION_RF => {}
                masslynx::MassLynxScanItem::TRANSPORT_RF => {}
                masslynx::MassLynxScanItem::SET_MASS => {
                    if ion.is_none() {
                        ion = Some(Default::default())
                    }
                    if let Some(ion) = ion.as_mut() {
                        ion.mz = val
                            .parse()
                            .unwrap_or_else(|e| panic!("Failed to parse SET_MASS: {e}"));
                    }
                }
                masslynx::MassLynxScanItem::COLLISION_ENERGY2 => {}
                masslynx::MassLynxScanItem::SET_MASS_CALL_SUPPORTED => {}
                masslynx::MassLynxScanItem::SET_MASS_CALIBRATED => {}
                masslynx::MassLynxScanItem::SONAR_ENABLED => {}
                masslynx::MassLynxScanItem::QUAD_START_MASS => {}
                masslynx::MassLynxScanItem::QUAD_STOP_MASS => {}
                masslynx::MassLynxScanItem::QUAD_PEAK_WIDTH => {}
                masslynx::MassLynxScanItem::REFERENCE_SCAN => {}
                masslynx::MassLynxScanItem::USE_LOCKMASS_CORRECTION => {}
                masslynx::MassLynxScanItem::LOCKMASS_CORRECTION => {}
                masslynx::MassLynxScanItem::USETEMP_CORRECTION => {}
                masslynx::MassLynxScanItem::TEMP_CORRECTION => {}
                masslynx::MassLynxScanItem::TEMP_COEFFICIENT => {}
                masslynx::MassLynxScanItem::FAIMS_COMPENSATION_VOLTAGE => {}
                masslynx::MassLynxScanItem::TIC_TRACE_A => {}
                masslynx::MassLynxScanItem::TIC_TRACE_B => {}
                masslynx::MassLynxScanItem::RAW_EE_CV => {}
                masslynx::MassLynxScanItem::RAW_EE_CE => {}
                masslynx::MassLynxScanItem::ACCURATE_MASS => {}
                masslynx::MassLynxScanItem::ACCURATE_MASS_FLAGS => {}
                masslynx::MassLynxScanItem::SCAN_ERROR_FLAG => {}
                masslynx::MassLynxScanItem::DRE_TRANSMISSION => {}
                masslynx::MassLynxScanItem::SCAN_PUSH_COUNT => {}
                masslynx::MassLynxScanItem::RAW_STAT_SWAVE_NORMALISATION_FACTOR => {}
                masslynx::MassLynxScanItem::MIN_DRIFT_TIME_CHANNEL => {}
                masslynx::MassLynxScanItem::MAX_DRIFT_TIME_CHANNEL => {}
                masslynx::MassLynxScanItem::TOTAL_ION_CURRENT => {
                    params.add_param(
                        Param::builder()
                            .name("total ion current")
                            .accession(1000285)
                            .controlled_vocabulary(MS)
                            .value(val.clone())
                            .unit(Unit::DetectorCounts)
                            .build(),
                    );
                }
                masslynx::MassLynxScanItem::BASE_PEAK_MASS => {
                    params.add_param(
                        Param::builder()
                            .name("base peak m/z")
                            .accession(1000504)
                            .controlled_vocabulary(MS)
                            .value(val.clone())
                            .unit(Unit::MZ)
                            .build(),
                    );
                }
                masslynx::MassLynxScanItem::BASE_PEAK_INTENSITY => {
                    params.add_param(
                        Param::builder()
                            .name("base peak intensity")
                            .accession(1000505)
                            .controlled_vocabulary(MS)
                            .value(val.clone())
                            .unit(Unit::DetectorCounts)
                            .build(),
                    );
                }
                masslynx::MassLynxScanItem::PEAKS_IN_SCAN => todo!(),
                masslynx::MassLynxScanItem::UNINITIALISED => todo!(),
            }
        }

        if let Some(ion) = ion {
            precursor.get_or_insert_default().add_ion(ion);
        }

        if let Some(activation) = activation {
            precursor.get_or_insert_default().activation = activation;
        }

        let descr = IonMobilityFrameDescription::new(
            id,
            index,
            ms_level,
            polarity,
            signal_continuity,
            params,
            acquisition,
            precursor,
            ion_mobility_unit,
        );

        let arrays: Result<(Vec<f64>, Vec<BinaryArrayMap>), ArrayRetrievalError> = cycle
            .signal
            .iter()
            .map(|s| -> Result<(f64, BinaryArrayMap), ArrayRetrievalError> {
                let mut buf = DataArray::from_name_type_size(
                    &ArrayType::MZArray,
                    BinaryDataArrayType::Float64,
                    s.mz_array.len() * BinaryDataArrayType::Float64.size_of(),
                );
                let view = buf.view_mut()?;
                view.extend(s.mz_array.iter().flat_map(|f| (*f as f64).to_le_bytes()));

                let mut buf2 = DataArray::from_name_type_size(
                    &ArrayType::IntensityArray,
                    BinaryDataArrayType::Float32,
                    s.mz_array.len() * BinaryDataArrayType::Float32.size_of(),
                );
                buf2.extend(&s.intensity_array)?;
                let mut arrays = BinaryArrayMap::new();
                arrays.add(buf);
                arrays.add(buf2);
                Ok((s.drift_time, arrays))
            })
            .collect();

        let (im_dimension, arrays) = arrays.ok()?;
        let arrays = BinaryArrayMap3D::from_ion_mobility_dimension_and_arrays(
            im_dimension,
            ArrayType::RawDriftTimeArray,
            Unit::Millisecond,
            arrays,
        );

        let frame = MultiLayerIonMobilityFrame::new(Some(arrays), None, None, descr);

        Some(frame)
    }
}

pub struct MassLynxSpectrumReaderType<
    C: CentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
    D: DeconvolutedCentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
    CF: FeatureLike<MZ, IonMobility>,
    DF: FeatureLike<Mass, IonMobility> + KnownCharge,
> {
    inner: MassLynxCycleReaderType<CF, DF>,
    index: usize,
    spectrum_index: OffsetIndex,
    _d: PhantomData<(C, D)>,
}

impl<
        C: CentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        CF: FeatureLike<MZ, IonMobility>,
        DF: FeatureLike<Mass, IonMobility> + KnownCharge,
    > ChromatogramSource for MassLynxSpectrumReaderType<C, D, CF, DF>
{
    fn get_chromatogram_by_id(&mut self, id: &str) -> Option<Chromatogram> {
        self.inner.get_chromatogram_by_id(id)
    }

    fn get_chromatogram_by_index(&mut self, index: usize) -> Option<Chromatogram> {
        self.inner.get_chromatogram_by_index(index)
    }
}

impl<
        C: CentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        CF: FeatureLike<MZ, IonMobility>,
        DF: FeatureLike<Mass, IonMobility> + KnownCharge,
    > MZFileReader<C, D, MultiLayerSpectrum<C, D>> for MassLynxSpectrumReaderType<C, D, CF, DF>
{
    fn construct_index_from_stream(&mut self) -> u64 {
        self.spectrum_index.len() as u64
    }

    #[allow(unused)]
    fn open_file(source: std::fs::File) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Cannot read a MassLynx dataset from an open file handle, only from a directory path",
        ))
    }

    fn open_path<P>(path: P) -> io::Result<Self>
    where
        P: Into<std::path::PathBuf> + Clone,
    {
        let path = path.into();
        let inner = MassLynxCycleReaderType::open_path(path)?;
        Ok(Self::new(inner))
    }
}

impl<
        C: CentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        CF: FeatureLike<MZ, IonMobility>,
        DF: FeatureLike<Mass, IonMobility> + KnownCharge,
    > Iterator for MassLynxSpectrumReaderType<C, D, CF, DF>
{
    type Item = MultiLayerSpectrum<C, D>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        let spec = self.get_spectrum(i);
        self.index += 1;
        spec
    }
}

impl<
        C: CentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        CF: FeatureLike<MZ, IonMobility>,
        DF: FeatureLike<Mass, IonMobility> + KnownCharge,
    > MassLynxSpectrumReaderType<C, D, CF, DF>
{
    fn build_index(&mut self) {
        self.inner
            .reader
            .index()
            .iter()
            .enumerate()
            .for_each(|(i, ent)| {
                self.spectrum_index
                    .insert(ent.native_id().into_boxed_str(), i as u64);
            });
    }

    fn new(inner: MassLynxCycleReaderType<CF, DF>) -> Self {
        let mut this = Self {
            inner,
            index: 0,
            spectrum_index: OffsetIndex::new("spectrum".into()),
            _d: PhantomData,
        };
        this.build_index();
        this
    }

    fn get_spectrum(&mut self, index: usize) -> Option<MultiLayerSpectrum<C, D>> {
        let spec: Spectrum = self.inner.reader.get_spectrum(index)?;
        let mut desc = SpectrumDescription::default();

        desc.id = spec.native_id();
        desc.index = spec.index;

        desc.signal_continuity = if spec.is_continuum {
            SignalContinuity::Profile
        } else {
            SignalContinuity::Centroid
        };

        desc.polarity = if spec.ion_mode.is_positive() {
            ScanPolarity::Positive
        } else {
            ScanPolarity::Negative
        };

        let func = &self.inner.reader.functions()[spec.function()];
        let scan = desc.acquisition.first_scan_mut().unwrap();
        scan.start_time = spec.time;
        scan.add_param(
            Param::builder()
                .accession(1000616)
                .controlled_vocabulary(MS)
                .name("preset scan configuration")
                .value(func.function + 1)
                .build(),
        );
        if let Some((lo, hi)) = func.scan_range {
            scan.scan_windows
                .push(ScanWindow::new(lo as f32, hi as f32))
        }

        if let Some(dt) = spec.drift_time {
            scan.add_param(
                Param::builder()
                    .name("ion mobility drift time")
                    .curie(curie!(MS:1002476))
                    .unit(Unit::Millisecond)
                    .value(dt)
                    .build(),
            );
        }

        let mut precursor: Option<Precursor> = None;
        let mut ion: Option<SelectedIon> = None;
        let mut activation: Option<Activation> = None;

        for (key, val) in spec.items.iter() {
            match key {
                masslynx::MassLynxScanItem::LINEAR_DETECTOR_VOLTAGE => {}
                masslynx::MassLynxScanItem::LINEAR_SENSITIVITY => {}
                masslynx::MassLynxScanItem::REFLECTRON_LENS_VOLTAGE => {}
                masslynx::MassLynxScanItem::REFLECTRON_DETECTOR_VOLTAGE => {}
                masslynx::MassLynxScanItem::REFLECTRON_SENSITIVITY => {}
                masslynx::MassLynxScanItem::LASER_REPETITION_RATE => {}
                masslynx::MassLynxScanItem::COURSE_LASER_CONTROL => {}
                masslynx::MassLynxScanItem::FINE_LASER_CONTROL => {}
                masslynx::MassLynxScanItem::LASERAIM_XPOS => {}
                masslynx::MassLynxScanItem::LASERAIM_YPOS => {}
                masslynx::MassLynxScanItem::NUM_SHOTS_SUMMED => {}
                masslynx::MassLynxScanItem::NUM_SHOTS_PERFORMED => {}
                masslynx::MassLynxScanItem::SEGMENT_NUMBER => {}
                masslynx::MassLynxScanItem::LCMP_TFM_WELL => {}
                masslynx::MassLynxScanItem::SEGMENT_TYPE => {}
                masslynx::MassLynxScanItem::SOURCE_REGION1 => {}
                masslynx::MassLynxScanItem::SOURCE_REGION2 => {}
                masslynx::MassLynxScanItem::REFLECTRON_FIELD_LENGTH => {}
                masslynx::MassLynxScanItem::REFLECTRON_LENGTH => {}
                masslynx::MassLynxScanItem::REFLECTRON_VOLT => {}
                masslynx::MassLynxScanItem::SAMPLE_PLATE_VOLT => {}
                masslynx::MassLynxScanItem::REFLECTRON_FIELD_LENGTH_ALT => {}
                masslynx::MassLynxScanItem::REFLECTRON_LENGTH_ALT => {}
                masslynx::MassLynxScanItem::PSD_STEP_MAJOR => {}
                masslynx::MassLynxScanItem::PSD_STEP_MINOR => {}
                masslynx::MassLynxScanItem::PSD_FACTOR_1 => {}
                masslynx::MassLynxScanItem::NEEDLE => {}
                masslynx::MassLynxScanItem::COUNTER_ELECTRODE_VOLTAGE => {}
                masslynx::MassLynxScanItem::SAMPLING_CONE_VOLTAGE => {}
                masslynx::MassLynxScanItem::SKIMMER_LENS => {}
                masslynx::MassLynxScanItem::SKIMMER => {}
                masslynx::MassLynxScanItem::PROBE_TEMPERATURE => {}
                masslynx::MassLynxScanItem::SOURCE_TEMPERATURE => {}
                masslynx::MassLynxScanItem::RF_VOLTAGE => {}
                masslynx::MassLynxScanItem::SOURCE_APERTURE => {}
                masslynx::MassLynxScanItem::SOURCE_CODE => {}
                masslynx::MassLynxScanItem::LM_RESOLUTION => {}
                masslynx::MassLynxScanItem::HM_RESOLUTION => {}
                masslynx::MassLynxScanItem::COLLISION_ENERGY => {
                    activation.get_or_insert_default().energy = val
                        .parse()
                        .unwrap_or_else(|e| panic!("Failed to parse COLLISION_ENERGY: {e}"))
                }
                masslynx::MassLynxScanItem::ION_ENERGY => {}
                masslynx::MassLynxScanItem::MULTIPLIER1 => {}
                masslynx::MassLynxScanItem::MULTIPLIER2 => {}
                masslynx::MassLynxScanItem::TRANSPORTDC => {}
                masslynx::MassLynxScanItem::TOF_APERTURE => {}
                masslynx::MassLynxScanItem::ACC_VOLTAGE => {}
                masslynx::MassLynxScanItem::STEERING => {}
                masslynx::MassLynxScanItem::FOCUS => {}
                masslynx::MassLynxScanItem::ENTRANCE => {}
                masslynx::MassLynxScanItem::GUARD => {}
                masslynx::MassLynxScanItem::TOF => {}
                masslynx::MassLynxScanItem::REFLECTRON => {}
                masslynx::MassLynxScanItem::COLLISION_RF => {}
                masslynx::MassLynxScanItem::TRANSPORT_RF => {}
                masslynx::MassLynxScanItem::SET_MASS => {
                    if ion.is_none() {
                        ion = Some(Default::default())
                    }
                    if let Some(ion) = ion.as_mut() {
                        ion.mz = val
                            .parse()
                            .unwrap_or_else(|e| panic!("Failed to parse SET_MASS: {e}"));
                    }
                }
                masslynx::MassLynxScanItem::COLLISION_ENERGY2 => {}
                masslynx::MassLynxScanItem::SET_MASS_CALL_SUPPORTED => {}
                masslynx::MassLynxScanItem::SET_MASS_CALIBRATED => {}
                masslynx::MassLynxScanItem::SONAR_ENABLED => {}
                masslynx::MassLynxScanItem::QUAD_START_MASS => {}
                masslynx::MassLynxScanItem::QUAD_STOP_MASS => {}
                masslynx::MassLynxScanItem::QUAD_PEAK_WIDTH => {}
                masslynx::MassLynxScanItem::REFERENCE_SCAN => {}
                masslynx::MassLynxScanItem::USE_LOCKMASS_CORRECTION => {}
                masslynx::MassLynxScanItem::LOCKMASS_CORRECTION => {}
                masslynx::MassLynxScanItem::USETEMP_CORRECTION => {}
                masslynx::MassLynxScanItem::TEMP_CORRECTION => {}
                masslynx::MassLynxScanItem::TEMP_COEFFICIENT => {}
                masslynx::MassLynxScanItem::FAIMS_COMPENSATION_VOLTAGE => {}
                masslynx::MassLynxScanItem::TIC_TRACE_A => {}
                masslynx::MassLynxScanItem::TIC_TRACE_B => {}
                masslynx::MassLynxScanItem::RAW_EE_CV => {}
                masslynx::MassLynxScanItem::RAW_EE_CE => {}
                masslynx::MassLynxScanItem::ACCURATE_MASS => {}
                masslynx::MassLynxScanItem::ACCURATE_MASS_FLAGS => {}
                masslynx::MassLynxScanItem::SCAN_ERROR_FLAG => {}
                masslynx::MassLynxScanItem::DRE_TRANSMISSION => {}
                masslynx::MassLynxScanItem::SCAN_PUSH_COUNT => {}
                masslynx::MassLynxScanItem::RAW_STAT_SWAVE_NORMALISATION_FACTOR => {}
                masslynx::MassLynxScanItem::MIN_DRIFT_TIME_CHANNEL => {}
                masslynx::MassLynxScanItem::MAX_DRIFT_TIME_CHANNEL => {}
                masslynx::MassLynxScanItem::TOTAL_ION_CURRENT => {
                    desc.params.add_param(
                        Param::builder()
                            .name("total ion current")
                            .accession(1000285)
                            .controlled_vocabulary(MS)
                            .value(val.clone())
                            .unit(Unit::DetectorCounts)
                            .build(),
                    );
                }
                masslynx::MassLynxScanItem::BASE_PEAK_MASS => {
                    desc.params.add_param(
                        Param::builder()
                            .name("base peak m/z")
                            .accession(1000504)
                            .controlled_vocabulary(MS)
                            .value(val.clone())
                            .unit(Unit::MZ)
                            .build(),
                    );
                }
                masslynx::MassLynxScanItem::BASE_PEAK_INTENSITY => {
                    desc.params.add_param(
                        Param::builder()
                            .name("base peak intensity")
                            .accession(1000505)
                            .controlled_vocabulary(MS)
                            .value(val.clone())
                            .unit(Unit::DetectorCounts)
                            .build(),
                    );
                }
                masslynx::MassLynxScanItem::PEAKS_IN_SCAN => todo!(),
                masslynx::MassLynxScanItem::UNINITIALISED => todo!(),
            }
        }

        if let Some(ion) = ion {
            precursor.get_or_insert_default().add_ion(ion);
        }

        if let Some(activation) = activation {
            precursor.get_or_insert_default().activation = activation;
        }

        desc.precursor = precursor;

        let arrays = if !matches!(self.inner.detail_level(), DetailLevel::MetadataOnly) {
            let mut mz_array = DataArray::from_name_type_size(
                &ArrayType::MZArray,
                BinaryDataArrayType::Float64,
                spec.mz_array.len() * BinaryDataArrayType::Float64.size_of(),
            );

            for value in spec.mz_array.iter().map(|v| *v as f64) {
                mz_array.push(value).unwrap();
            }

            let mut intensity_array = DataArray::from_name_type_size(
                &ArrayType::IntensityArray,
                BinaryDataArrayType::Float32,
                BinaryDataArrayType::Float32.size_of() * spec.intensity_array.len(),
            );

            intensity_array.extend(&spec.intensity_array).unwrap();

            let mut arrays = BinaryArrayMap::new();
            arrays.add(mz_array);
            arrays.add(intensity_array);
            Some(arrays)
        } else {
            None
        };

        Some(MultiLayerSpectrum::new(desc, arrays, None, None))
    }
}

impl<
        C: CentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        D: DeconvolutedCentroidLike + BuildArrayMapFrom + BuildFromArrayMap,
        CF: FeatureLike<MZ, IonMobility>,
        DF: FeatureLike<Mass, IonMobility> + KnownCharge,
    > SpectrumSource<C, D, MultiLayerSpectrum<C, D>> for MassLynxSpectrumReaderType<C, D, CF, DF>
{
    fn reset(&mut self) {
        self.index = 0;
    }

    fn detail_level(&self) -> &DetailLevel {
        &self.inner.detail_level
    }

    fn set_detail_level(&mut self, detail_level: DetailLevel) {
        self.inner.set_detail_level(detail_level);
    }

    fn get_spectrum_by_id(&mut self, id: &str) -> Option<MultiLayerSpectrum<C, D>> {
        let offset = self.spectrum_index.get(id)?;
        let spec = self.inner.reader.get_spectrum(offset as usize)?;
        let mut desc = SpectrumDescription::default();

        desc.id = spec.native_id();
        desc.index = spec.index;

        desc.signal_continuity = if spec.is_continuum {
            SignalContinuity::Profile
        } else {
            SignalContinuity::Centroid
        };

        desc.polarity = if spec.ion_mode.is_positive() {
            ScanPolarity::Positive
        } else {
            ScanPolarity::Negative
        };

        let scan = desc.acquisition.first_scan_mut().unwrap();
        scan.start_time = spec.time;

        if let Some(dt) = spec.drift_time {
            scan.add_param(
                Param::builder()
                    .name("ion mobility drift time")
                    .curie(curie!(MS:1002476))
                    .unit(Unit::Millisecond)
                    .value(dt)
                    .build(),
            );
        }

        None
    }

    fn get_spectrum_by_index(&mut self, index: usize) -> Option<MultiLayerSpectrum<C, D>> {
        self.get_spectrum(index)
    }

    fn get_index(&self) -> &OffsetIndex {
        &self.spectrum_index
    }

    fn set_index(&mut self, index: OffsetIndex) {
        self.spectrum_index = index
    }
}
