//! A subset of the raw C API is declared for MassLynxRaw.
//!
//! Not all signatures have been converted, and not all have "safe"
//! bindings.
//!
//! See [`base`](crate::base) for safer wrappers.
//! See [`constants`](crate::constants) for various enums that are used to name
//! things.
#![allow(unused)]
use std::ffi::{c_char, c_float, c_int, c_uint, c_void};

use crate::constants::{
    MassLynxAcquisitionType, MassLynxBaseType, MassLynxFunctionType, MassLynxHeaderItem,
    MassLynxIonMode, MassLynxScanItem,
};

#[allow(unused)]
pub type CMassLynxAcquisition = *mut c_void;
pub type CMassLynxParameters = *mut c_void;
pub type CMassLynxBaseReader = *mut c_void;
pub type CMassLynxBaseProcessor = *mut c_void;
#[allow(unused)]
pub type CMassLynxRawWriter = *mut c_void;
#[allow(unused)]
pub type CMassLynxSampleList = *mut c_void;
// void(__stdcall *ProgressCallBack)(void* pObject, const int& percent);
pub type ProgressCallBack = Option<unsafe extern "stdcall" fn(*const c_void, *const c_int)>;

#[link(name = "MassLynxRaw", kind = "static")]
extern "stdcall" {
    pub fn releaseMemory(memory: *const c_void) -> c_int;
    pub fn getErrorMessage(nErrorCode: c_int, ppErrorMessage: *const *const c_char) -> c_int;

    pub fn getVersionInfo(ppVersion: *mut *const c_char) -> c_int;

    // Base reader
    pub fn createRawReaderFromPath(
        path: *const c_char,
        mlRawReader: *mut CMassLynxBaseReader,
        nType: MassLynxBaseType,
    ) -> c_int;

    pub fn createRawReaderFromReader(
        mlSourceRawReader: CMassLynxBaseReader,
        mlRawReader: *mut CMassLynxBaseReader,
        nType: MassLynxBaseType,
    ) -> c_int;

    pub fn destroyRawReader(mlRawReader: CMassLynxBaseReader) -> c_int;

    // Information reader
    pub fn getFunctionCount(mlInfoReader: CMassLynxBaseReader, pFunctions: *const c_uint) -> c_int;
    pub fn getScanCount(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        pScans: *const c_uint,
    ) -> c_int;
    pub fn getAcquisitionMassRange(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichSegment: c_int,
        lowMass: *const c_float,
        highMass: *const c_float,
    ) -> c_int;
    pub fn getAcquisitionTimeRange(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        startTime: *const c_float,
        endTime: *const c_float,
    ) -> c_int;
    pub fn getFunctionType(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        pFunctionType: *const MassLynxFunctionType,
    ) -> c_int;
    pub fn getFunctionTypeString(
        mlInfoReader: CMassLynxBaseReader,
        functionType: MassLynxFunctionType,
        chFunctionType: *const *const c_char,
    ) -> c_int;
    pub fn isContinuum(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        bContinuum: *const c_char,
    ) -> c_int;
    pub fn getIonMode(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        ionMode: *const MassLynxIonMode,
    ) -> c_int;
    pub fn getIonModeString(
        mlInfoReader: CMassLynxBaseReader,
        ionMode: MassLynxIonMode,
        chIonMode: *const *const c_char,
    ) -> c_int;
    pub fn getRetentionTime(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        fRT: *mut c_float,
    ) -> c_int;
    pub fn getDriftTime(
        mlInfoReader: CMassLynxBaseReader,
        nWhichDrift: c_int,
        fRT: *mut c_float,
    ) -> c_int;
    pub fn getDriftTime_CCS(
        mlInfoReader: CMassLynxBaseReader,
        ccs: c_float,
        mass: c_float,
        charge: c_int,
        driftTime: *const c_float,
    ) -> c_int;
    pub fn getCollisionalCrossSection(
        mlInfoReader: CMassLynxBaseReader,
        driftTime: c_float,
        mass: c_float,
        charge: c_int,
        fCCS: *mut c_float,
    ) -> c_int;
    pub fn getDriftScanCount(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        pScans: *const c_uint,
    ) -> c_int;
    pub fn getMRMCount(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        pMRMs: *mut c_int,
    ) -> c_int;
    pub fn isLockMassCorrected(
        mlInfoReader: CMassLynxBaseReader,
        pIsApplied: *const c_char,
    ) -> c_int;
    pub fn canLockMassCorrect(mlInfoReader: CMassLynxBaseReader, pCanApply: *const c_char)
        -> c_int;
    pub fn getLockMassFunction(
        mlRawReader: CMassLynxBaseReader,
        hasLockmass: *mut c_char,
        whichFunction: *mut c_int,
    ) -> c_int;
    pub fn getIndexRange(
        mlRawReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        preCursorMass: c_float,
        preCursorTolerance: c_float,
        pStartIndex: *const c_int,
        pEndIndex: *const c_int,
    ) -> c_int;
    pub fn getHeaderItemValue(
        mlRawReader: CMassLynxBaseReader,
        pItems: *const MassLynxHeaderItem,
        nItems: c_int,
        pParameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getAcquisitionInfo(
        mlInfoReader: CMassLynxBaseReader,
        parameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getScanItemValue(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        pItems: *const MassLynxScanItem,
        nItems: c_int,
        pParameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getScanItemName(
        mlInfoReader: CMassLynxBaseReader,
        pItems: *const MassLynxScanItem,
        nItems: c_int,
        pParameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getScanItemsInFunction(
        mlInfoReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        parameters: CMassLynxParameters,
    ) -> c_int;

    // Scan Reader functions
    pub fn readScan(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        ppMasses: *const *const c_float,
        ppIntensities: *const *const c_float,
        pSize: *const c_int,
    ) -> c_int;
    pub fn readScanFlags(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        pMasses: *const *const c_float,
        pIntensities: *const *const c_float,
        pFlags: *const *const c_char,
        pSize: *const c_int,
    ) -> c_int;
    pub fn readDriftScan(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        nWhichDrift: c_int,
        ppMasses: *const *const c_float,
        ppIntensities: *const *const c_float,
        pSize: *const c_int,
    ) -> c_int;
    pub fn readDaughterScan(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        ppMasses: *const *const c_float,
        ppIntensities: *const *const c_float,
        ppProductMasses: *const *const c_float,
        pSize: *const c_int,
        pProductSize: *const c_int,
    ) -> c_int;
    pub fn readDriftScanIndex(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        nWhichDrift: c_int,
        ppMasses: *const *const c_int,
        ppIntensities: *const *const c_float,
        pSize: *const c_int,
    ) -> c_int;
    pub fn readDriftScanFlagsIndex(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        nWhichDrift: c_int,
        ppMasses: *const *const c_int,
        ppIntensities: *const *const c_float,
        pFlags: *const *const c_char,
        pSize: *const c_int,
    ) -> c_int;
    pub fn getDriftMassScale(
        mlRawScanreader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        nWhichScan: c_int,
        ppMasses: *const *const c_float,
        pSize: *const c_int,
        pOffset: *const c_int,
    ) -> c_int;

    pub fn createParameters(mlParameters: *const CMassLynxParameters) -> c_int;
    pub fn createParametersFromParameters(
        mlSourceParameters: CMassLynxParameters,
        mlParameters: *mut CMassLynxParameters,
    ) -> c_int;
    pub fn destroyParameters(mlParameters: CMassLynxParameters) -> c_int;
    pub fn setParameterValue(
        mlParameters: CMassLynxParameters,
        nKey: c_int,
        pValue: *const c_char,
    ) -> c_int;
    pub fn getParameterValue(
        mlParameters: CMassLynxParameters,
        nKey: c_int,
        ppValue: *const *const c_char,
    ) -> c_int;
    pub fn getParameterKeys(
        mlParameters: CMassLynxParameters,
        ppKeys: *const *const c_int,
        pSize: *const c_int,
    ) -> c_int;

    // Chromatogram functions
    pub fn readBPIChromatogram(
        mlChromatogramReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        ppTimes: *const *const c_float,
        ppIntensities: *const *const c_float,
        pSize: *const c_int,
    ) -> c_int;

    pub fn readTICChromatogram(
        mlChromatogramReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        ppTimes: *const *const c_float,
        ppIntensities: *const *const c_float,
        pSize: *const c_int,
    ) -> c_int;

    pub fn readMassChromatograms(
        mlChromatogramReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        massList: *const c_float,
        massListSize: c_int,
        ppTimes: *const *const c_float,
        ppIntensities: *const *const c_float,
        massWindow: c_float,
        bDaughters: c_char,
        pSize: *const c_int,
    ) -> c_int;
    pub fn readSonarMassChromatogram(
        mlChromatogramReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        preCursorMass: c_float,
        pMass: c_float,
        ppTimes: *const *const c_float,
        ppIntensities: *const *const c_float,
        precursorMassWindow: c_float,
        massWindow: c_float,
        pSize: *const c_int,
    ) -> c_int;

    pub fn readMRMChromatograms(
        mlChromatogramReader: CMassLynxBaseReader,
        nWhichFunction: c_int,
        mrmList: *const c_int,
        nMRM: c_int,
        ppTimes: *const *const c_float,
        ppIntensities: *const *const c_float,
        pSize: *const c_int,
    ) -> c_int;

    pub fn readMobillogram(
        mlChromatogramReader: CMassLynxBaseReader,
        whichFunction: c_int,
        startScan: c_int,
        endScan: c_int,
        startMass: c_float,
        endMass: c_float,
        ppDriftBins: *const *const c_int,
        ppIntensities: *const *const f32,
        pSize: *const c_int,
    ) -> c_int;

    // Base processor
    pub fn createRawProcessor(
        mlRawProcessor: *const CMassLynxBaseProcessor,
        nType: MassLynxBaseType,
        pCallback: ProgressCallBack,
        pCaller: *const c_void,
    ) -> c_int;
    pub fn destroyRawProcessor(mlRawProcessor: CMassLynxBaseProcessor) -> c_int;
    pub fn getProcessorMessage(
        mlRawProcessor: CMassLynxBaseProcessor,
        nCode: c_int,
        ppMessage: *const *const c_char,
    ) -> c_int;
    pub fn setRawReader(
        mlRawProcessor: CMassLynxBaseProcessor,
        mlRawReader: CMassLynxBaseReader,
    ) -> c_int;
    pub fn setRawPath(mlRawProcessor: CMassLynxBaseProcessor, path: *const c_char) -> c_int;
    pub fn setProcessorCallBack(
        mlRawProcessor: CMassLynxBaseProcessor,
        pCallback: ProgressCallBack,
        pCaller: *const c_void,
    ) -> c_int;

    // Lock mass processor
    pub fn setLockMassParameters(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        pParameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getLockMassParameter(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        ppParameters: *const *const c_char,
    ) -> c_int;
    pub fn lockMassCorrect(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        pApplied: *const c_char,
    ) -> c_int;
    pub fn removeLockMassCorrection(mlLockMassProcessor: CMassLynxBaseProcessor) -> c_int;
    pub fn getLockMassCandidates(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        ppMasses: *const *const c_float,
        ppIntensities: *const *const c_float,
        nSize: *const c_int,
    ) -> c_int;
    pub fn LMP_isLockMassCorrected(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        applied: *const c_int,
    ) -> c_int;

    pub fn LMP_canLockMassCorrect(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        canApply: *const c_int,
    ) -> c_int;

    pub fn getLockMassCorrection(
        mlLockMassProcessor: CMassLynxBaseProcessor,
        retentionTime: c_float,
        pGain: *const c_float,
    ) -> c_int;

    // Analog reader functions
    pub fn getChannelCount(mlAnalogReader: CMassLynxBaseReader, nChannels: *mut c_int) -> c_int;
    pub fn readChannel(
        mlAnalogReader: CMassLynxBaseReader,
        nWhichChannel: c_int,
        pTimes: *const *const c_float,
        pIntensities: *const *const c_float,
        pSize: *mut c_int,
    ) -> c_int;
    pub fn getChannelDesciption(
        mlAnalogReader: CMassLynxBaseReader,
        nWhichChannel: c_int,
        ppDescription: *const *const c_char,
    ) -> c_int;
    pub fn getChannelUnits(
        mlAnalogReader: CMassLynxBaseReader,
        nWhichChannel: c_int,
        ppUnits: *const *const c_char,
    ) -> c_int;

    /// Scan processor functions
    pub fn getScan(
        mlScanProcessor: CMassLynxBaseProcessor,
        ppMasses: *const *const c_float,
        ppIntensities: *const *const c_float,
        nSize: *mut c_int,
    ) -> c_int;
    pub fn setScan(
        mlScanProcessor: CMassLynxBaseProcessor,
        pMasses: *const c_float,
        pIntensities: *const c_float,
        nMassSize: c_int,
        nIntensitySize: c_int,
    ) -> c_int;

    // combine
    pub fn combineScan(
        mlScanProcessor: CMassLynxBaseProcessor,
        whichFunction: c_int,
        startScan: c_int,
        endScan: c_int,
    ) -> c_int;
    pub fn combineDriftScan(
        mlSpectrumProcessor: CMassLynxBaseProcessor,
        whichFunction: c_int,
        startScan: c_int,
        endScan: c_int,
        startDrift: c_int,
        endDrift: c_int,
    ) -> c_int;

    // smooth
    pub fn smoothScan(mlScanProcessor: CMassLynxBaseProcessor) -> c_int;
    pub fn setSmoothParameter(
        mlScanProcessor: CMassLynxBaseProcessor,
        pParameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getSmoothParameter(
        mlScanProcessor: CMassLynxBaseProcessor,
        parameters: CMassLynxParameters,
    ) -> c_int;

    // centroid
    pub fn centroidScan(mlScanProcessor: CMassLynxBaseProcessor) -> c_int;
    pub fn setCentroidParameter(
        mlScanProcessor: CMassLynxBaseProcessor,
        parameters: CMassLynxParameters,
    ) -> c_int;
    pub fn getCentroidParameter(
        mlScanProcessor: CMassLynxBaseProcessor,
        parameters: CMassLynxParameters,
    ) -> c_int;

    // theshold
    pub fn thresholdScan(mlScanProcessor: CMassLynxBaseProcessor) -> c_int;
    pub fn setThresholdParameter(
        mlScanProcessor: CMassLynxBaseProcessor,
        pParameters: CMassLynxParameters,
    ) -> c_int;
}
