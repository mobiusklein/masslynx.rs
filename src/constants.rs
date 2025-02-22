#![allow(unused, non_camel_case_types)]

use std::fmt::Debug;
use std::ffi::c_int;

const TYPE_BASE: u32 = 1;
const ION_MODE_BASE: u32 = 100;
const FUNCTION_TYPE_BASE: u32 = 200;
const HEADER_ITEM_BASE: u32 = 300;
const SCAN_ITEM_BASE: u32 = 400;
const SAMPLELIST_ITEM_BASE: u32 = 700;
const BATCH_ITEM_BASE: u32 = 900;
const LOCKMASS_ITEM_BASE: u32 = 1000;
const LOCKMASS_COMPOUND_BASE: u32 = 1050;
const FUNCTION_DEFINITION_BASE: u32 = 1100;
const ANALOG_PARAMETER_BASE: u32 = 1200;
const ANALOG_TYPE_BASE: u32 = 1250;
const AUTOLYNX_STATUS_BASE: u32 = 1300;
const AUTOLYNX_SETTINGS_BASE: u32 = 1350;
const CENTROID_ITEM_BASE: u32 = 1400;
const SMOOTH_ITEM_BASE: u32 = 1450;
const SMOOTH_TYPE_BASE: u32 = 1500;
const THESHOLD_ITEM_BASE: u32 = 1550;
const THESHOLD_TYPE_BASE: u32 = 1600;
const ACQUISITION_PARAMETER_BASE: u32 = 1650;
const ACQUISITION_TYPE_BASE: u32 = 1700;
const STATUS_TYPE_BASE: u32 = 1750;
const DDA_TYPE_BASE: u32 = 1800;
const SCAN_TYPE_BASE: u32 = 1850;
const DDA_ISOLATION_WINDOW_PARAMETER_BASE: u32 = 1900;
const DDA_PARAMETER_BASE: u32 = 1950;

pub trait AsMassLynxItemKey: TryFrom<i32> + Copy + Debug + Eq + std::hash::Hash {
    fn as_key(&self) -> c_int;
}

impl AsMassLynxItemKey for i32 {
    fn as_key(&self) -> c_int {
        *self
    }
}

macro_rules! impl_as_key {
    ($t:ty) => {
        impl AsMassLynxItemKey for $t {
            fn as_key(&self) -> c_int {
                *self as c_int
            }
        }
    };
    ($($ts:ty, )+) => {
        $(
            impl_as_key!($ts);
        )+
    };
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MassLynxBaseType {
    SCAN = TYPE_BASE,
    INFO = TYPE_BASE + 1,
    CHROM = TYPE_BASE + 2,
    ANALOG = TYPE_BASE + 3,
    LOCKMASS = TYPE_BASE + 4,
    CENTROID = TYPE_BASE + 5,
    DDA = TYPE_BASE + 6,
    MSE = TYPE_BASE + 7,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MassLynxIonMode {
    EI_POS = ION_MODE_BASE,
    EI_NEG = ION_MODE_BASE + 1,
    CI_POS = ION_MODE_BASE + 2,
    CI_NEG = ION_MODE_BASE + 3,
    FB_POS = ION_MODE_BASE + 4,
    FB_NEG = ION_MODE_BASE + 5,
    TS_POS = ION_MODE_BASE + 6,
    TS_NEG = ION_MODE_BASE + 7,
    ES_POS = ION_MODE_BASE + 8,
    ES_NEG = ION_MODE_BASE + 9,
    AI_POS = ION_MODE_BASE + 10,
    AI_NEG = ION_MODE_BASE + 11,
    LD_POS = ION_MODE_BASE + 12,
    LD_NEG = ION_MODE_BASE + 13,
    #[default]
    UNINITIALISED = ION_MODE_BASE + 99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MassLynxFunctionType { // ProteoWizard classifications
    /// FunctionType_Scan, |  Standard MS scanning function
    MS = FUNCTION_TYPE_BASE,
    /// FunctionType_SIR, |  Selected ion recording
    SIR = 1 + FUNCTION_TYPE_BASE,
    /// FunctionType_Delay, |  No longer supported
    DLY = 2 + FUNCTION_TYPE_BASE,
    /// FunctionType_Concatenated, |  No longer supported
    CAT = 3 + FUNCTION_TYPE_BASE,
    /// FunctionType_Off, |  No longer supported
    OFF = 4 + FUNCTION_TYPE_BASE,
    /// FunctionType_Parents, |  MSMS Parent scan
    PAR = 5 + FUNCTION_TYPE_BASE,
    /// FunctionType_Daughters, |  MSMS Daughter scan
    DAU = 6 + FUNCTION_TYPE_BASE,
    /// FunctionType_Neutral_Loss, |  MSMS Neutral Loss
    NL = 7 + FUNCTION_TYPE_BASE,
    /// FunctionType_Neutral_Gain, |  MSMS Neutral Gain
    NG = 8 + FUNCTION_TYPE_BASE,
    /// FunctionType_MRM, |  Multiple Reaction Monitoring
    MRM = 9 + FUNCTION_TYPE_BASE,
    /// FunctionType_Q1F, |  Special function used on Quattro IIs for scanning MS1 (Q1) but uses the final detector
    Q1F = 10 + FUNCTION_TYPE_BASE,
    /// FunctionType_MS2, |  Special function used on triple quads for scanning MS2. Used for calibration experiments.
    MS2 = 11 + FUNCTION_TYPE_BASE,
    /// FunctionType_Diode_Array, |  Diode array type function
    DAD = 12 + FUNCTION_TYPE_BASE,
    /// FunctionType_TOF, |  TOF
    TOF = 13 + FUNCTION_TYPE_BASE,
    /// FunctionType_TOF_PSD, |  TOF Post Source Decay type function
    PSD = 14 + FUNCTION_TYPE_BASE,
    /// FunctionType_TOF_Survey, |  QTOF MS Survey scan
    TOFS = 15 + FUNCTION_TYPE_BASE,
    /// FunctionType_TOF_Daughter, |  QTOF MSMS scan
    TOFD = 16 + FUNCTION_TYPE_BASE,
    /// FunctionType_MALDI_TOF, |  Maldi-Tof function
    MTOF = 17 + FUNCTION_TYPE_BASE,
    /// FunctionType_TOF_MS, |  QTOF MS scan
    TOFM = 18 + FUNCTION_TYPE_BASE,
    /// FunctionType_TOF_Parent, |  QTOF Parent scan
    TOFP = 19 + FUNCTION_TYPE_BASE,
    /// FunctionType_Voltage_Scan, |  AutoSpec Voltage Scan
    ASVS = 20 + FUNCTION_TYPE_BASE,
    /// FunctionType_Magnetic_Scan, |  AutoSpec Magnet Scan
    ASMS = 21 + FUNCTION_TYPE_BASE,
    /// FunctionType_Voltage_SIR, |  AutoSpec Voltage SIR
    ASVSIR = 22 + FUNCTION_TYPE_BASE,
    /// FunctionType_Magnetic_SIR, |  AutoSpec Magnet SIR
    ASMSIR = 23 + FUNCTION_TYPE_BASE,
    /// FunctionType_Auto_Daughters, |  Quad Automated daughter scanning
    QUADD = 24 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_B_E_Scan, |  AutoSpec_B_E_Scan
    ASBE = 25 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_B2_E_Scan, |  AutoSpec_B2_E_Scan
    ASB2E = 26 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_CNL_Scan, |  AutoSpec_CNL_Scan
    ASCNL = 27 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_MIKES_Scan, |  AutoSpec_MIKES_Scan
    ASMIKES = 28 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_MRM, |  AutoSpec_MRM
    ASMRM = 29 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_NRMS_Scan, |  AutoSpec_NRMS_Scan
    ASNRMS = 30 + FUNCTION_TYPE_BASE,
    /// FunctionType_AutoSpec_Q_MRM_Quad, |  AutoSpec_Q_MRM_Quad
    ASMRMQ = 31 + FUNCTION_TYPE_BASE,
    UNINITIALISED = FUNCTION_TYPE_BASE + 99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MassLynxHeaderItem {
    VERSION = HEADER_ITEM_BASE,
    ACQUIRED_NAME = 1 + HEADER_ITEM_BASE,
    ACQUIRED_DATE = 2 + HEADER_ITEM_BASE,
    ACQUIRED_TIME = 3 + HEADER_ITEM_BASE,
    JOB_CODE = 4 + HEADER_ITEM_BASE,
    TASK_CODE = 5 + HEADER_ITEM_BASE,
    USER_NAME = 6 + HEADER_ITEM_BASE,
    INSTRUMENT = 7 + HEADER_ITEM_BASE,
    CONDITIONS = 8 + HEADER_ITEM_BASE,
    LAB_NAME = 9 + HEADER_ITEM_BASE,
    SAMPLE_DESCRIPTION = 10 + HEADER_ITEM_BASE,
    SOLVENT_DELAY = 11 + HEADER_ITEM_BASE,
    SUBMITTER = 12 + HEADER_ITEM_BASE,
    SAMPLE_ID = 13 + HEADER_ITEM_BASE,
    BOTTLE_NUMBER = 14 + HEADER_ITEM_BASE,
    ANALOG_CH1_OFFSET = 15 + HEADER_ITEM_BASE,
    ANALOG_CH2_OFFSET = 16 + HEADER_ITEM_BASE,
    ANALOG_CH3_OFFSET = 17 + HEADER_ITEM_BASE,
    ANALOG_CH4_OFFSET = 18 + HEADER_ITEM_BASE,
    CAL_MS1_STATIC = 19 + HEADER_ITEM_BASE,
    CAL_MS2_STATIC = 20 + HEADER_ITEM_BASE,
    CAL_MS1_STATIC_PARAMS = 21 + HEADER_ITEM_BASE,
    CAL_MS1_DYNAMIC_PARAMS = 22 + HEADER_ITEM_BASE,
    CAL_MS2_STATIC_PARAMS = 23 + HEADER_ITEM_BASE,
    CAL_MS2_DYNAMIC_PARAMS = 24 + HEADER_ITEM_BASE,
    CAL_MS1_FAST_PARAMS = 25 + HEADER_ITEM_BASE,
    CAL_MS2_FAST_PARAMS = 26 + HEADER_ITEM_BASE,
    CAL_TIME = 27 + HEADER_ITEM_BASE,
    CAL_DATE = 28 + HEADER_ITEM_BASE,
    CAL_TEMPERATURE = 29 + HEADER_ITEM_BASE,
    INLET_METHOD = 30 + HEADER_ITEM_BASE,
    SPARE1 = 31 + HEADER_ITEM_BASE,
    SPARE2 = 32 + HEADER_ITEM_BASE,
    SPARE3 = 33 + HEADER_ITEM_BASE,
    SPARE4 = 34 + HEADER_ITEM_BASE,
    SPARE5 = 35 + HEADER_ITEM_BASE,
}

impl MassLynxHeaderItem {
    pub fn iter() -> impl Iterator<Item = Self> {
        (HEADER_ITEM_BASE..).map_while(|i| {
            (i as i32).try_into().ok()
        })
    }
}

impl TryFrom<i32> for MassLynxHeaderItem {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value as u32 {
            x if x  == Self::VERSION as u32 => Self::VERSION,
            x if x  == Self::ACQUIRED_NAME as u32 => Self::ACQUIRED_NAME,
            x if x  == Self::ACQUIRED_DATE as u32 => Self::ACQUIRED_DATE,
            x if x  == Self::ACQUIRED_TIME as u32 => Self::ACQUIRED_TIME,
            x if x  == Self::JOB_CODE as u32 => Self::JOB_CODE,
            x if x  == Self::TASK_CODE as u32 => Self::TASK_CODE,
            x if x  == Self::USER_NAME as u32 => Self::USER_NAME,
            x if x  == Self::INSTRUMENT as u32 => Self::INSTRUMENT,
            x if x  == Self::CONDITIONS as u32 => Self::CONDITIONS,
            x if x  == Self::LAB_NAME as u32 => Self::LAB_NAME,
            x if x  == Self::SAMPLE_DESCRIPTION as u32 => Self::SAMPLE_DESCRIPTION,
            x if x  == Self::SOLVENT_DELAY as u32 => Self::SOLVENT_DELAY,
            x if x  == Self::SUBMITTER as u32 => Self::SUBMITTER,
            x if x  == Self::SAMPLE_ID as u32 => Self::SAMPLE_ID,
            x if x  == Self::BOTTLE_NUMBER as u32 => Self::BOTTLE_NUMBER,
            x if x  == Self::ANALOG_CH1_OFFSET as u32 => Self::ANALOG_CH1_OFFSET,
            x if x  == Self::ANALOG_CH2_OFFSET as u32 => Self::ANALOG_CH2_OFFSET,
            x if x  == Self::ANALOG_CH3_OFFSET as u32 => Self::ANALOG_CH3_OFFSET,
            x if x  == Self::ANALOG_CH4_OFFSET as u32 => Self::ANALOG_CH4_OFFSET,
            x if x  == Self::CAL_MS1_STATIC as u32 => Self::CAL_MS1_STATIC,
            x if x  == Self::CAL_MS2_STATIC as u32 => Self::CAL_MS2_STATIC,
            x if x  == Self::CAL_MS1_STATIC_PARAMS as u32 => Self::CAL_MS1_STATIC_PARAMS,
            x if x  == Self::CAL_MS1_DYNAMIC_PARAMS as u32 => Self::CAL_MS1_DYNAMIC_PARAMS,
            x if x  == Self::CAL_MS2_STATIC_PARAMS as u32 => Self::CAL_MS2_STATIC_PARAMS,
            x if x  == Self::CAL_MS2_DYNAMIC_PARAMS as u32 => Self::CAL_MS2_DYNAMIC_PARAMS,
            x if x  == Self::CAL_MS1_FAST_PARAMS as u32 => Self::CAL_MS1_FAST_PARAMS,
            x if x  == Self::CAL_MS2_FAST_PARAMS as u32 => Self::CAL_MS2_FAST_PARAMS,
            x if x  == Self::CAL_TIME as u32 => Self::CAL_TIME,
            x if x  == Self::CAL_DATE as u32 => Self::CAL_DATE,
            x if x  == Self::CAL_TEMPERATURE as u32 => Self::CAL_TEMPERATURE,
            x if x  == Self::INLET_METHOD as u32 => Self::INLET_METHOD,
            x if x  == Self::SPARE1 as u32 => Self::SPARE1,
            x if x  == Self::SPARE2 as u32 => Self::SPARE2,
            x if x  == Self::SPARE3 as u32 => Self::SPARE3,
            x if x  == Self::SPARE4 as u32 => Self::SPARE4,
            x if x  == Self::SPARE5 as u32 => Self::SPARE5,
            _ => return Err(format!("Cannot convert {value} into MassLynxHeaderItem"))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MassLynxScanItem {
    LINEAR_DETECTOR_VOLTAGE = SCAN_ITEM_BASE,
    LINEAR_SENSITIVITY = SCAN_ITEM_BASE + 1,
    REFLECTRON_LENS_VOLTAGE = SCAN_ITEM_BASE + 2,
    REFLECTRON_DETECTOR_VOLTAGE = SCAN_ITEM_BASE + 3,
    REFLECTRON_SENSITIVITY = SCAN_ITEM_BASE + 4,
    LASER_REPETITION_RATE = SCAN_ITEM_BASE + 5,
    COURSE_LASER_CONTROL = SCAN_ITEM_BASE + 6,
    FINE_LASER_CONTROL = SCAN_ITEM_BASE + 7,
    LASERAIM_XPOS = SCAN_ITEM_BASE + 8,
    LASERAIM_YPOS = SCAN_ITEM_BASE + 9,
    NUM_SHOTS_SUMMED = SCAN_ITEM_BASE + 10,
    NUM_SHOTS_PERFORMED = SCAN_ITEM_BASE + 11,
    SEGMENT_NUMBER = SCAN_ITEM_BASE + 12,
    LCMP_TFM_WELL = SCAN_ITEM_BASE + 13,
    SEGMENT_TYPE = SCAN_ITEM_BASE + 14,
    SOURCE_REGION1 = SCAN_ITEM_BASE + 15,
    SOURCE_REGION2 = SCAN_ITEM_BASE + 16,
    REFLECTRON_FIELD_LENGTH = SCAN_ITEM_BASE + 17,
    REFLECTRON_LENGTH = SCAN_ITEM_BASE + 18,
    REFLECTRON_VOLT = SCAN_ITEM_BASE + 19,
    SAMPLE_PLATE_VOLT = SCAN_ITEM_BASE + 20,
    REFLECTRON_FIELD_LENGTH_ALT = SCAN_ITEM_BASE + 21,
    REFLECTRON_LENGTH_ALT = SCAN_ITEM_BASE + 22,
    PSD_STEP_MAJOR = SCAN_ITEM_BASE + 23,
    PSD_STEP_MINOR = SCAN_ITEM_BASE + 24,
    PSD_FACTOR_1 = SCAN_ITEM_BASE + 25,
    NEEDLE = SCAN_ITEM_BASE + 49,
    COUNTER_ELECTRODE_VOLTAGE = SCAN_ITEM_BASE + 50,
    SAMPLING_CONE_VOLTAGE = SCAN_ITEM_BASE + 51,
    SKIMMER_LENS = SCAN_ITEM_BASE + 52,
    SKIMMER = SCAN_ITEM_BASE + 53,
    PROBE_TEMPERATURE = SCAN_ITEM_BASE + 54,
    SOURCE_TEMPERATURE = SCAN_ITEM_BASE + 55,
    RF_VOLTAGE = SCAN_ITEM_BASE + 56,
    SOURCE_APERTURE = SCAN_ITEM_BASE + 57,
    SOURCE_CODE = SCAN_ITEM_BASE + 58,
    LM_RESOLUTION = SCAN_ITEM_BASE + 59,
    HM_RESOLUTION = SCAN_ITEM_BASE + 60,
    COLLISION_ENERGY = SCAN_ITEM_BASE + 61,
    ION_ENERGY = SCAN_ITEM_BASE + 62,
    MULTIPLIER1 = SCAN_ITEM_BASE + 63,
    MULTIPLIER2 = SCAN_ITEM_BASE + 64,
    TRANSPORTDC = SCAN_ITEM_BASE + 65,
    TOF_APERTURE = SCAN_ITEM_BASE + 66,
    ACC_VOLTAGE = SCAN_ITEM_BASE + 67,
    STEERING = SCAN_ITEM_BASE + 68,
    FOCUS = SCAN_ITEM_BASE + 69,
    ENTRANCE = SCAN_ITEM_BASE + 70,
    GUARD = SCAN_ITEM_BASE + 71,
    TOF = SCAN_ITEM_BASE + 72,
    REFLECTRON = SCAN_ITEM_BASE + 73,
    COLLISION_RF = SCAN_ITEM_BASE + 74,
    TRANSPORT_RF = SCAN_ITEM_BASE + 75,
    SET_MASS = SCAN_ITEM_BASE + 76,
    COLLISION_ENERGY2 = SCAN_ITEM_BASE + 77,
    SET_MASS_CALL_SUPPORTED = SCAN_ITEM_BASE + 78,
    SET_MASS_CALIBRATED = SCAN_ITEM_BASE + 79,
    SONAR_ENABLED = SCAN_ITEM_BASE + 80,
    QUAD_START_MASS = SCAN_ITEM_BASE + 81,
    QUAD_STOP_MASS = SCAN_ITEM_BASE + 82,
    QUAD_PEAK_WIDTH = SCAN_ITEM_BASE + 83,
    REFERENCE_SCAN = SCAN_ITEM_BASE + 99,
    USE_LOCKMASS_CORRECTION = SCAN_ITEM_BASE + 100,
    LOCKMASS_CORRECTION = SCAN_ITEM_BASE + 101,
    USETEMP_CORRECTION = SCAN_ITEM_BASE + 102,
    TEMP_CORRECTION = SCAN_ITEM_BASE + 103,
    TEMP_COEFFICIENT = SCAN_ITEM_BASE + 104,
    FAIMS_COMPENSATION_VOLTAGE = SCAN_ITEM_BASE + 105,
    TIC_TRACE_A = SCAN_ITEM_BASE + 106,
    TIC_TRACE_B = SCAN_ITEM_BASE + 107,
    RAW_EE_CV = SCAN_ITEM_BASE + 108,
    RAW_EE_CE = SCAN_ITEM_BASE + 110,
    ACCURATE_MASS = SCAN_ITEM_BASE + 111,
    ACCURATE_MASS_FLAGS = SCAN_ITEM_BASE + 112,
    SCAN_ERROR_FLAG = SCAN_ITEM_BASE + 113,
    DRE_TRANSMISSION = SCAN_ITEM_BASE + 114,
    SCAN_PUSH_COUNT = SCAN_ITEM_BASE + 115,
    RAW_STAT_SWAVE_NORMALISATION_FACTOR = SCAN_ITEM_BASE + 116,
    MIN_DRIFT_TIME_CHANNEL = SCAN_ITEM_BASE + 121,
    MAX_DRIFT_TIME_CHANNEL = SCAN_ITEM_BASE + 122,
    TOTAL_ION_CURRENT = SCAN_ITEM_BASE + 251,
    BASE_PEAK_MASS = SCAN_ITEM_BASE + 252,
    BASE_PEAK_INTENSITY = SCAN_ITEM_BASE + 253,
    PEAKS_IN_SCAN = SCAN_ITEM_BASE + 254,
    UNINITIALISED = SCAN_ITEM_BASE + 298,
}

impl TryFrom<i32> for MassLynxScanItem {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value as u32 {
            x if x == Self::LINEAR_DETECTOR_VOLTAGE as u32 => Self::LINEAR_DETECTOR_VOLTAGE,
            x if x == Self::LINEAR_SENSITIVITY as u32 => Self::LINEAR_SENSITIVITY,
            x if x == Self::REFLECTRON_LENS_VOLTAGE as u32 => Self::REFLECTRON_LENS_VOLTAGE,
            x if x == Self::REFLECTRON_DETECTOR_VOLTAGE as u32 => Self::REFLECTRON_DETECTOR_VOLTAGE,
            x if x == Self::REFLECTRON_SENSITIVITY as u32 => Self::REFLECTRON_SENSITIVITY,
            x if x == Self::LASER_REPETITION_RATE as u32 => Self::LASER_REPETITION_RATE,
            x if x == Self::COURSE_LASER_CONTROL as u32 => Self::COURSE_LASER_CONTROL,
            x if x == Self::FINE_LASER_CONTROL as u32 => Self::FINE_LASER_CONTROL,
            x if x == Self::LASERAIM_XPOS as u32 => Self::LASERAIM_XPOS,
            x if x == Self::LASERAIM_YPOS as u32 => Self::LASERAIM_YPOS,
            x if x == Self::NUM_SHOTS_SUMMED as u32 => Self::NUM_SHOTS_SUMMED,
            x if x == Self::NUM_SHOTS_PERFORMED as u32 => Self::NUM_SHOTS_PERFORMED,
            x if x == Self::SEGMENT_NUMBER as u32 => Self::SEGMENT_NUMBER,
            x if x == Self::LCMP_TFM_WELL as u32 => Self::LCMP_TFM_WELL,
            x if x == Self::SEGMENT_TYPE as u32 => Self::SEGMENT_TYPE,
            x if x == Self::SOURCE_REGION1 as u32 => Self::SOURCE_REGION1,
            x if x == Self::SOURCE_REGION2 as u32 => Self::SOURCE_REGION2,
            x if x == Self::REFLECTRON_FIELD_LENGTH as u32 => Self::REFLECTRON_FIELD_LENGTH,
            x if x == Self::REFLECTRON_LENGTH as u32 => Self::REFLECTRON_LENGTH,
            x if x == Self::REFLECTRON_VOLT as u32 => Self::REFLECTRON_VOLT,
            x if x == Self::SAMPLE_PLATE_VOLT as u32 => Self::SAMPLE_PLATE_VOLT,
            x if x == Self::REFLECTRON_FIELD_LENGTH_ALT as u32 => Self::REFLECTRON_FIELD_LENGTH_ALT,
            x if x == Self::REFLECTRON_LENGTH_ALT as u32 => Self::REFLECTRON_LENGTH_ALT,
            x if x == Self::PSD_STEP_MAJOR as u32 => Self::PSD_STEP_MAJOR,
            x if x == Self::PSD_STEP_MINOR as u32 => Self::PSD_STEP_MINOR,
            x if x == Self::PSD_FACTOR_1 as u32 => Self::PSD_FACTOR_1,
            x if x == Self::NEEDLE as u32 => Self::NEEDLE,
            x if x == Self::COUNTER_ELECTRODE_VOLTAGE as u32 => Self::COUNTER_ELECTRODE_VOLTAGE,
            x if x == Self::SAMPLING_CONE_VOLTAGE as u32 => Self::SAMPLING_CONE_VOLTAGE,
            x if x == Self::SKIMMER_LENS as u32 => Self::SKIMMER_LENS,
            x if x == Self::SKIMMER as u32 => Self::SKIMMER,
            x if x == Self::PROBE_TEMPERATURE as u32 => Self::PROBE_TEMPERATURE,
            x if x == Self::SOURCE_TEMPERATURE as u32 => Self::SOURCE_TEMPERATURE,
            x if x == Self::RF_VOLTAGE as u32 => Self::RF_VOLTAGE,
            x if x == Self::SOURCE_APERTURE as u32 => Self::SOURCE_APERTURE,
            x if x == Self::SOURCE_CODE as u32 => Self::SOURCE_CODE,
            x if x == Self::LM_RESOLUTION as u32 => Self::LM_RESOLUTION,
            x if x == Self::HM_RESOLUTION as u32 => Self::HM_RESOLUTION,
            x if x == Self::COLLISION_ENERGY as u32 => Self::COLLISION_ENERGY,
            x if x == Self::ION_ENERGY as u32 => Self::ION_ENERGY,
            x if x == Self::MULTIPLIER1 as u32 => Self::MULTIPLIER1,
            x if x == Self::MULTIPLIER2 as u32 => Self::MULTIPLIER2,
            x if x == Self::TRANSPORTDC as u32 => Self::TRANSPORTDC,
            x if x == Self::TOF_APERTURE as u32 => Self::TOF_APERTURE,
            x if x == Self::ACC_VOLTAGE as u32 => Self::ACC_VOLTAGE,
            x if x == Self::STEERING as u32 => Self::STEERING,
            x if x == Self::FOCUS as u32 => Self::FOCUS,
            x if x == Self::ENTRANCE as u32 => Self::ENTRANCE,
            x if x == Self::GUARD as u32 => Self::GUARD,
            x if x == Self::TOF as u32 => Self::TOF,
            x if x == Self::REFLECTRON as u32 => Self::REFLECTRON,
            x if x == Self::COLLISION_RF as u32 => Self::COLLISION_RF,
            x if x == Self::TRANSPORT_RF as u32 => Self::TRANSPORT_RF,
            x if x == Self::SET_MASS as u32 => Self::SET_MASS,
            x if x == Self::COLLISION_ENERGY2 as u32 => Self::COLLISION_ENERGY2,
            x if x == Self::SET_MASS_CALL_SUPPORTED as u32 => Self::SET_MASS_CALL_SUPPORTED,
            x if x == Self::SET_MASS_CALIBRATED as u32 => Self::SET_MASS_CALIBRATED,
            x if x == Self::SONAR_ENABLED as u32 => Self::SONAR_ENABLED,
            x if x == Self::QUAD_START_MASS as u32 => Self::QUAD_START_MASS,
            x if x == Self::QUAD_STOP_MASS as u32 => Self::QUAD_STOP_MASS,
            x if x == Self::QUAD_PEAK_WIDTH as u32 => Self::QUAD_PEAK_WIDTH,
            x if x == Self::REFERENCE_SCAN as u32 => Self::REFERENCE_SCAN,
            x if x == Self::USE_LOCKMASS_CORRECTION as u32 => Self::USE_LOCKMASS_CORRECTION,
            x if x == Self::LOCKMASS_CORRECTION as u32 => Self::LOCKMASS_CORRECTION,
            x if x == Self::USETEMP_CORRECTION as u32 => Self::USETEMP_CORRECTION,
            x if x == Self::TEMP_CORRECTION as u32 => Self::TEMP_CORRECTION,
            x if x == Self::TEMP_COEFFICIENT as u32 => Self::TEMP_COEFFICIENT,
            x if x == Self::FAIMS_COMPENSATION_VOLTAGE as u32 => Self::FAIMS_COMPENSATION_VOLTAGE,
            x if x == Self::TIC_TRACE_A as u32 => Self::TIC_TRACE_A,
            x if x == Self::TIC_TRACE_B as u32 => Self::TIC_TRACE_B,
            x if x == Self::RAW_EE_CV as u32 => Self::RAW_EE_CV,
            x if x == Self::RAW_EE_CE as u32 => Self::RAW_EE_CE,
            x if x == Self::ACCURATE_MASS as u32 => Self::ACCURATE_MASS,
            x if x == Self::ACCURATE_MASS_FLAGS as u32 => Self::ACCURATE_MASS_FLAGS,
            x if x == Self::SCAN_ERROR_FLAG as u32 => Self::SCAN_ERROR_FLAG,
            x if x == Self::DRE_TRANSMISSION as u32 => Self::DRE_TRANSMISSION,
            x if x == Self::SCAN_PUSH_COUNT as u32 => Self::SCAN_PUSH_COUNT,
            x if x == Self::RAW_STAT_SWAVE_NORMALISATION_FACTOR as u32 => Self::RAW_STAT_SWAVE_NORMALISATION_FACTOR,
            x if x == Self::MIN_DRIFT_TIME_CHANNEL as u32 => Self::MIN_DRIFT_TIME_CHANNEL,
            x if x == Self::MAX_DRIFT_TIME_CHANNEL as u32 => Self::MAX_DRIFT_TIME_CHANNEL,
            x if x == Self::TOTAL_ION_CURRENT as u32 => Self::TOTAL_ION_CURRENT,
            x if x == Self::BASE_PEAK_MASS as u32 => Self::BASE_PEAK_MASS,
            x if x == Self::BASE_PEAK_INTENSITY as u32 => Self::BASE_PEAK_INTENSITY,
            x if x == Self::PEAKS_IN_SCAN as u32 => Self::PEAKS_IN_SCAN,
            x if x == Self::UNINITIALISED as u32 => Self::UNINITIALISED,
            _ => return Err(format!("Cannot map {value} to MassLynxScanItem"))
        })
    }
}

const FILE_NAME: u32 = 700;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MassLynxSampleListItem {
    FILE_NAME = FILE_NAME,
    FILE_TEXT = FILE_NAME + 1,
    MS_FILE = FILE_NAME + 2,
    MS_TUNE_FILE = FILE_NAME + 3,
    INLET_FILE = FILE_NAME + 4,
    INLET_PRERUN = FILE_NAME + 5,
    INLET_POSTRUN = FILE_NAME + 6,
    INLET_SWITCH = FILE_NAME + 7,
    AUTO_FILE = FILE_NAME + 8,
    PROCESS = FILE_NAME + 9,
    PROCESS_PARAMS = FILE_NAME + 10,
    PROCESS_OPTIONS = FILE_NAME + 11,
    ACQU_PROCESS_FILE = FILE_NAME + 12,
    ACQU_PROCESS_PARAMS = FILE_NAME + 13,
    ACQU_PROCESS_OPTIONS = FILE_NAME + 14,
    PROCESS_ACTION = FILE_NAME + 15,
    SAMPLE_LOCATION = FILE_NAME + 16,
    SAMPLE_GROUP = FILE_NAME + 17,
    JOB = FILE_NAME + 18,
    TASK = FILE_NAME + 19,
    USER = FILE_NAME + 20,
    SUBMITTER = FILE_NAME + 21,
    CONDITIONS = FILE_NAME + 22,
    TYPE = FILE_NAME + 23,
    CONTROL = FILE_NAME + 24,
    ID = FILE_NAME + 25,
    CONC_A = FILE_NAME + 26,
    CONC_B = FILE_NAME + 27,
    CONC_C = FILE_NAME + 28,
    CONC_D = FILE_NAME + 29,
    CONC_E = FILE_NAME + 30,
    CONC_F = FILE_NAME + 31,
    CONC_G = FILE_NAME + 32,
    CONC_H = FILE_NAME + 33,
    CONC_I = FILE_NAME + 34,
    CONC_J = FILE_NAME + 35,
    CONC_K = FILE_NAME + 36,
    CONC_L = FILE_NAME + 37,
    CONC_M = FILE_NAME + 38,
    CONC_N = FILE_NAME + 39,
    CONC_O = FILE_NAME + 40,
    CONC_P = FILE_NAME + 41,
    CONC_Q = FILE_NAME + 42,
    CONC_R = FILE_NAME + 43,
    CONC_S = FILE_NAME + 44,
    CONC_T = FILE_NAME + 45,
    WAVELENGTH_A = FILE_NAME + 46,
    WAVELENGTH_B = FILE_NAME + 47,
    WAVELENGTH_C = FILE_NAME + 48,
    WAVELENGTH_D = FILE_NAME + 49,
    WAVELENGTH_E = FILE_NAME + 50,
    WAVELENGTH_F = FILE_NAME + 51,
    WAVELENGTH_G = FILE_NAME + 52,
    WAVELENGTH_H = FILE_NAME + 53,
    WAVELENGTH_I = FILE_NAME + 54,
    WAVELENGTH_J = FILE_NAME + 55,
    MASS_A = FILE_NAME + 56,
    MASS_B = FILE_NAME + 57,
    MASS_C = FILE_NAME + 58,
    MASS_D = FILE_NAME + 59,
    MASS_E = FILE_NAME + 60,
    MASS_F = FILE_NAME + 61,
    MASS_G = FILE_NAME + 62,
    MASS_H = FILE_NAME + 63,
    MASS_I = FILE_NAME + 64,
    MASS_J = FILE_NAME + 65,
    MASS_K = FILE_NAME + 66,
    MASS_L = FILE_NAME + 67,
    MASS_M = FILE_NAME + 68,
    MASS_N = FILE_NAME + 69,
    MASS_O = FILE_NAME + 70,
    MASS_P = FILE_NAME + 71,
    MASS_Q = FILE_NAME + 72,
    MASS_R = FILE_NAME + 73,
    MASS_S = FILE_NAME + 74,
    MASS_T = FILE_NAME + 75,
    MASS_U = FILE_NAME + 76,
    MASS_V = FILE_NAME + 77,
    MASS_W = FILE_NAME + 78,
    MASS_X = FILE_NAME + 79,
    MASS_Y = FILE_NAME + 80,
    MASS_Z = FILE_NAME + 81,
    MASS_AA = FILE_NAME + 82,
    MASS_BB = FILE_NAME + 83,
    MASS_CC = FILE_NAME + 84,
    MASS_DD = FILE_NAME + 85,
    FRACTION_FILE = FILE_NAME + 86,
    FRACTION_1 = FILE_NAME + 87,
    FRACTION_2 = FILE_NAME + 88,
    FRACTION_3 = FILE_NAME + 89,
    FRACTION_4 = FILE_NAME + 90,
    FRACTION_5 = FILE_NAME + 91,
    FRACTION_6 = FILE_NAME + 92,
    FRACTION_7 = FILE_NAME + 93,
    FRACTION_8 = FILE_NAME + 94,
    FRACTION_9 = FILE_NAME + 95,
    FRACTION_10 = FILE_NAME + 96,
    FRACTION_BOOLEAN_LOGIC = FILE_NAME + 97,
    FRACTION_START = FILE_NAME + 98,
    INJ_VOL = FILE_NAME + 99,
    STOCK_DIL = FILE_NAME + 100,
    USER_DIVISOR_1 = FILE_NAME + 101,
    USER_FACTOR_1 = FILE_NAME + 102,
    USER_FACTOR_2 = FILE_NAME + 103,
    USER_FACTOR_3 = FILE_NAME + 104,
    SPARE_1 = FILE_NAME + 105,
    SPARE_2 = FILE_NAME + 106,
    SPARE_3 = FILE_NAME + 107,
    SPARE_4 = FILE_NAME + 108,
    SPARE_5 = FILE_NAME + 109,
    HPLC_FILE = FILE_NAME + 110,
    QUAN_REF = FILE_NAME + 111,
    AUTO_ADDITION = FILE_NAME + 112,
    MOLFILE = FILE_NAME + 113,
    SUBJECTTEXT = FILE_NAME + 114,
    SUBJECTTIME = FILE_NAME + 115,
    METH_DB = FILE_NAME + 116,
    CURVE_DB = FILE_NAME + 117,
}

impl TryFrom<i32> for MassLynxSampleListItem {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let this = Self::ACQU_PROCESS_FILE;
        Ok(match value as u32 {
            FILE_NAME => Self::FILE_NAME,
            x if x == FILE_NAME + 1 => Self::FILE_TEXT,
            x if x == FILE_NAME + 2 => Self::MS_FILE,
            x if x == FILE_NAME + 3 => Self::MS_TUNE_FILE,
            x if x == FILE_NAME + 4 => Self::INLET_FILE,
            x if x == FILE_NAME + 5 => Self::INLET_PRERUN,
            x if x == FILE_NAME + 6 => Self::INLET_POSTRUN,
            x if x == FILE_NAME + 7 => Self::INLET_SWITCH,
            x if x == FILE_NAME + 8 => Self::AUTO_FILE,
            x if x == FILE_NAME + 9 => Self::PROCESS,
            x if x == FILE_NAME + 10 => Self::PROCESS_PARAMS,
            x if x == FILE_NAME + 11 => Self::PROCESS_OPTIONS,
            x if x == FILE_NAME + 12 => Self::ACQU_PROCESS_FILE,
            x if x == FILE_NAME + 13 => Self::ACQU_PROCESS_PARAMS,
            x if x == FILE_NAME + 14 => Self::ACQU_PROCESS_OPTIONS,
            x if x == FILE_NAME + 15 => Self::PROCESS_ACTION,
            x if x == FILE_NAME + 16 => Self::SAMPLE_LOCATION,
            x if x == FILE_NAME + 17 => Self::SAMPLE_GROUP,
            x if x == FILE_NAME + 18 => Self::JOB,
            x if x == FILE_NAME + 19 => Self::TASK,
            x if x == FILE_NAME + 20 => Self::USER,
            x if x == FILE_NAME + 21 => Self::SUBMITTER,
            x if x == FILE_NAME + 22 => Self::CONDITIONS,
            x if x == FILE_NAME + 23 => Self::TYPE,
            x if x == FILE_NAME + 24 => Self::CONTROL,
            x if x == FILE_NAME + 25 => Self::ID,
            x if x == FILE_NAME + 26 => Self::CONC_A,
            x if x == FILE_NAME + 27 => Self::CONC_B,
            x if x == FILE_NAME + 28 => Self::CONC_C,
            x if x == FILE_NAME + 29 => Self::CONC_D,
            x if x == FILE_NAME + 30 => Self::CONC_E,
            x if x == FILE_NAME + 31 => Self::CONC_F,
            x if x == FILE_NAME + 32 => Self::CONC_G,
            x if x == FILE_NAME + 33 => Self::CONC_H,
            x if x == FILE_NAME + 34 => Self::CONC_I,
            x if x == FILE_NAME + 35 => Self::CONC_J,
            x if x == FILE_NAME + 36 => Self::CONC_K,
            x if x == FILE_NAME + 37 => Self::CONC_L,
            x if x == FILE_NAME + 38 => Self::CONC_M,
            x if x == FILE_NAME + 39 => Self::CONC_N,
            x if x == FILE_NAME + 40 => Self::CONC_O,
            x if x == FILE_NAME + 41 => Self::CONC_P,
            x if x == FILE_NAME + 42 => Self::CONC_Q,
            x if x == FILE_NAME + 43 => Self::CONC_R,
            x if x == FILE_NAME + 44 => Self::CONC_S,
            x if x == FILE_NAME + 45 => Self::CONC_T,
            x if x == FILE_NAME + 46 => Self::WAVELENGTH_A,
            x if x == FILE_NAME + 47 => Self::WAVELENGTH_B,
            x if x == FILE_NAME + 48 => Self::WAVELENGTH_C,
            x if x == FILE_NAME + 49 => Self::WAVELENGTH_D,
            x if x == FILE_NAME + 50 => Self::WAVELENGTH_E,
            x if x == FILE_NAME + 51 => Self::WAVELENGTH_F,
            x if x == FILE_NAME + 52 => Self::WAVELENGTH_G,
            x if x == FILE_NAME + 53 => Self::WAVELENGTH_H,
            x if x == FILE_NAME + 54 => Self::WAVELENGTH_I,
            x if x == FILE_NAME + 55 => Self::WAVELENGTH_J,
            x if x == FILE_NAME + 56 => Self::MASS_A,
            x if x == FILE_NAME + 57 => Self::MASS_B,
            x if x == FILE_NAME + 58 => Self::MASS_C,
            x if x == FILE_NAME + 59 => Self::MASS_D,
            x if x == FILE_NAME + 60 => Self::MASS_E,
            x if x == FILE_NAME + 61 => Self::MASS_F,
            x if x == FILE_NAME + 62 => Self::MASS_G,
            x if x == FILE_NAME + 63 => Self::MASS_H,
            x if x == FILE_NAME + 64 => Self::MASS_I,
            x if x == FILE_NAME + 65 => Self::MASS_J,
            x if x == FILE_NAME + 66 => Self::MASS_K,
            x if x == FILE_NAME + 67 => Self::MASS_L,
            x if x == FILE_NAME + 68 => Self::MASS_M,
            x if x == FILE_NAME + 69 => Self::MASS_N,
            x if x == FILE_NAME + 70 => Self::MASS_O,
            x if x == FILE_NAME + 71 => Self::MASS_P,
            x if x == FILE_NAME + 72 => Self::MASS_Q,
            x if x == FILE_NAME + 73 => Self::MASS_R,
            x if x == FILE_NAME + 74 => Self::MASS_S,
            x if x == FILE_NAME + 75 => Self::MASS_T,
            x if x == FILE_NAME + 76 => Self::MASS_U,
            x if x == FILE_NAME + 77 => Self::MASS_V,
            x if x == FILE_NAME + 78 => Self::MASS_W,
            x if x == FILE_NAME + 79 => Self::MASS_X,
            x if x == FILE_NAME + 80 => Self::MASS_Y,
            x if x == FILE_NAME + 81 => Self::MASS_Z,
            x if x == FILE_NAME + 82 => Self::MASS_AA,
            x if x == FILE_NAME + 83 => Self::MASS_BB,
            x if x == FILE_NAME + 84 => Self::MASS_CC,
            x if x == FILE_NAME + 85 => Self::MASS_DD,
            x if x == FILE_NAME + 86 => Self::FRACTION_FILE,
            x if x == FILE_NAME + 87 => Self::FRACTION_1,
            x if x == FILE_NAME + 88 => Self::FRACTION_2,
            x if x == FILE_NAME + 89 => Self::FRACTION_3,
            x if x == FILE_NAME + 90 => Self::FRACTION_4,
            x if x == FILE_NAME + 91 => Self::FRACTION_5,
            x if x == FILE_NAME + 92 => Self::FRACTION_6,
            x if x == FILE_NAME + 93 => Self::FRACTION_7,
            x if x == FILE_NAME + 94 => Self::FRACTION_8,
            x if x == FILE_NAME + 95 => Self::FRACTION_9,
            x if x == FILE_NAME + 96 => Self::FRACTION_10,
            x if x == FILE_NAME + 97 => Self::FRACTION_BOOLEAN_LOGIC,
            x if x == FILE_NAME + 98 => Self::FRACTION_START,
            x if x == FILE_NAME + 99 => Self::INJ_VOL,
            x if x == FILE_NAME + 100 => Self::STOCK_DIL,
            x if x == FILE_NAME + 101 => Self::USER_DIVISOR_1,
            x if x == FILE_NAME + 102 => Self::USER_FACTOR_1,
            x if x == FILE_NAME + 103 => Self::USER_FACTOR_2,
            x if x == FILE_NAME + 104 => Self::USER_FACTOR_3,
            x if x == FILE_NAME + 105 => Self::SPARE_1,
            x if x == FILE_NAME + 106 => Self::SPARE_2,
            x if x == FILE_NAME + 107 => Self::SPARE_3,
            x if x == FILE_NAME + 108 => Self::SPARE_4,
            x if x == FILE_NAME + 109 => Self::SPARE_5,
            x if x == FILE_NAME + 110 => Self::HPLC_FILE,
            x if x == FILE_NAME + 111 => Self::QUAN_REF,
            x if x == FILE_NAME + 112 => Self::AUTO_ADDITION,
            x if x == FILE_NAME + 113 => Self::MOLFILE,
            x if x == FILE_NAME + 114 => Self::SUBJECTTEXT,
            x if x == FILE_NAME + 115 => Self::SUBJECTTIME,
            x if x == FILE_NAME + 116 => Self::METH_DB,
            x if x == FILE_NAME + 117 => Self::CURVE_DB,
            _ => return Err(format!("No mapping for {value} to MassLynxSampleListItem"))
        })
    }
}

impl_as_key!(MassLynxHeaderItem, MassLynxScanItem, MassLynxSampleListItem, );


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MassLynxBatchItem {
	SAMPLELIST_NAME = BATCH_ITEM_BASE,
	FIRST_SAMPLE = BATCH_ITEM_BASE + 1,
	LAST_SAMPLE = BATCH_ITEM_BASE + 2,
	CURRENT_SAMPLE = BATCH_ITEM_BASE + 3,
	BATCH_USER_NAME = BATCH_ITEM_BASE + 4
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MassLynxAcquisitionType {
	DDA = ACQUISITION_TYPE_BASE,
	MSE = ACQUISITION_TYPE_BASE + 1,
	HDDDA = ACQUISITION_TYPE_BASE + 2,
	HDMSE = ACQUISITION_TYPE_BASE + 3,
	SONAR = ACQUISITION_TYPE_BASE + 4,
	UNKNOWN = ACQUISITION_TYPE_BASE + 48,
	UNINITIALISED = ACQUISITION_TYPE_BASE + 49
}

impl TryFrom<i32> for MassLynxAcquisitionType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value as u32 {
            x if x == Self::DDA as u32 => Self::DDA,
            x if x == Self::MSE as u32 => Self::MSE,
            x if x == Self::HDDDA as u32 => Self::HDDDA,
            x if x == Self::HDMSE as u32 => Self::HDMSE,
            x if x == Self::SONAR as u32 => Self::SONAR,
            x if x == Self::UNKNOWN as u32 => Self::UNKNOWN,
            x if x == Self::UNINITIALISED as u32 => Self::UNINITIALISED,
            _ => return Err(format!("Cannot convert {value} into MassLynxAcquisitionType")),
        })
    }
}

impl_as_key!(MassLynxAcquisitionType);


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum MassLynxScanType {
	MS1 = SCAN_TYPE_BASE,
	MS2 = SCAN_TYPE_BASE + 1,
	UNINITIALISED = SCAN_TYPE_BASE + 9
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum LockMassParameter {
    MASS = LOCKMASS_ITEM_BASE,
    TOLERANCE = LOCKMASS_ITEM_BASE + 1,
    FORCE = LOCKMASS_ITEM_BASE + 2,
}

impl TryFrom<i32> for LockMassParameter {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value as u32 {
            LOCKMASS_ITEM_BASE => Self::MASS,
            x if x == Self::TOLERANCE as u32 => Self::TOLERANCE,
            x if x == Self::FORCE as u32 => Self::FORCE,
            _ => return Err(format!("Could not convert {value} to LockMassParameter"))
        })
    }
}

impl_as_key!(LockMassParameter);


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FunctionDefinition {
    CONTINUUM = FUNCTION_DEFINITION_BASE,
    IONMODE = FUNCTION_DEFINITION_BASE + 1,
    FUNCTIONTYPE = FUNCTION_DEFINITION_BASE + 2,
    STARTMASS = FUNCTION_DEFINITION_BASE + 3,
    ENDMASS = FUNCTION_DEFINITION_BASE + 4,
    CDT_SCANS = FUNCTION_DEFINITION_BASE + 5,
    SAMPLINGFREQUENCY = FUNCTION_DEFINITION_BASE + 6,
    LTEFF = FUNCTION_DEFINITION_BASE + 7,
    VEFF = FUNCTION_DEFINITION_BASE + 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AnalogParameter {
    DESCRIPTION = ANALOG_PARAMETER_BASE + 1,
    UNITS = ANALOG_PARAMETER_BASE + 2,
    TYPE = ANALOG_PARAMETER_BASE + 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AnalogTraceType {
    ANALOG = ANALOG_TYPE_BASE,
    ELSD = ANALOG_TYPE_BASE + 1,
    READBACK = ANALOG_TYPE_BASE + 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AutoLynxStatus {
    QUEUED = AUTOLYNX_STATUS_BASE,
    PROCESSED = AUTOLYNX_STATUS_BASE + 1,
    FAILED = AUTOLYNX_STATUS_BASE + 2,
    NOTFOUND = AUTOLYNX_STATUS_BASE + 3,
    UNINITIALISED = AUTOLYNX_STATUS_BASE + 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum CentroidParameter
{
	RESOLUTION = CENTROID_ITEM_BASE
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MassLynxDDAIndexDetail {
	RT = DDA_TYPE_BASE,
	FUNCTION = DDA_TYPE_BASE + 1,
	START_SCAN = DDA_TYPE_BASE + 2,
	END_SCAN = DDA_TYPE_BASE + 3,
	SCAN_TYPE= DDA_TYPE_BASE + 4,
	SET_MASS = DDA_TYPE_BASE + 5,
	PRECURSOR_MASS = DDA_TYPE_BASE + 6
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DDAIsolationWindowParameter {
	LOWEROFFSET = DDA_ISOLATION_WINDOW_PARAMETER_BASE,
	UPPEROFFSET = DDA_ISOLATION_WINDOW_PARAMETER_BASE + 1
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum SmoothParameter {
	NUMBER = SMOOTH_ITEM_BASE,
	WIDTH = SMOOTH_ITEM_BASE + 1,
	SMOOTHTYPE = SMOOTH_ITEM_BASE + 2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum SmoothType {
	MEAN = SMOOTH_TYPE_BASE,
	MEDIAN = SMOOTH_TYPE_BASE + 1,
	SAVITZKY_GOLAY = SMOOTH_TYPE_BASE + 2
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum ThresholdParameter {
	VALUE = THESHOLD_ITEM_BASE,
	TYPE = THESHOLD_ITEM_BASE + 1
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ThresholdType {
	ABSOLUTE_THESHOLD = THESHOLD_TYPE_BASE,
	RELATIVE_THESHOLD = THESHOLD_TYPE_BASE + 1
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum AcquisitionParameter {
	TYPE = ACQUISITION_PARAMETER_BASE,
	LOCKMASS = ACQUISITION_PARAMETER_BASE + 1,
	MS1 = ACQUISITION_PARAMETER_BASE + 2,
	MS2 = ACQUISITION_PARAMETER_BASE + 3,
	PRECURSOR_MASS_START = ACQUISITION_PARAMETER_BASE + 4,
	PRECURSOR_MASS_END = ACQUISITION_PARAMETER_BASE + 5,
	FUNCTIONS = ACQUISITION_PARAMETER_BASE + 6,
	SAMPLINGFREQUENCY = ACQUISITION_PARAMETER_BASE + 7,
	LTEFF = ACQUISITION_PARAMETER_BASE + 8,
	VEFF = ACQUISITION_PARAMETER_BASE + 9,
	RESOLUTION = ACQUISITION_PARAMETER_BASE + 10
}

impl TryFrom<i32> for AcquisitionParameter {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value as u32 {
            x if x == Self::TYPE as u32 => Self::TYPE,
            x if x == Self::LOCKMASS as u32 => Self::LOCKMASS,
            x if x == Self::MS1 as u32 => Self::MS1,
            x if x == Self::MS2 as u32 => Self::MS2,
            x if x == Self::PRECURSOR_MASS_START as u32 => Self::PRECURSOR_MASS_START,
            x if x == Self::PRECURSOR_MASS_END as u32 => Self::PRECURSOR_MASS_END,
            x if x == Self::FUNCTIONS as u32 => Self::FUNCTIONS,
            x if x == Self::SAMPLINGFREQUENCY as u32 => Self::SAMPLINGFREQUENCY,
            x if x == Self::LTEFF as u32 => Self::LTEFF,
            x if x == Self::VEFF as u32 => Self::VEFF,
            x if x == Self::RESOLUTION as u32 => Self::RESOLUTION,
            _ => return Err(format!("Cannot convert {value} into AcquisitionParameter")),
        })
    }
}

impl_as_key!(AcquisitionParameter);