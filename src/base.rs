use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_char, c_float, c_int, c_uint, c_void, CStr, CString};
use std::fmt::Display;
use std::hash::Hash;
use std::path::Path;
use std::{mem, ptr};

use log::trace;

use crate::constants::MassLynxHeaderItem;
use crate::{
    constants::{
        AsMassLynxItemKey, MassLynxBaseType, MassLynxFunctionType, MassLynxIonMode,
        MassLynxScanItem,
    },
    ffi,
};

macro_rules! fficall {
    ($task:tt) => {
        #[allow(unused_braces)]
        let code = unsafe { $task };
        if code != 0 {
            return Err(Self::mass_lynx_error_for_code(code));
        }
    };
}

#[derive(Debug, Default, Clone)]
pub struct MassLynxError {
    pub error_code: i32,
    pub message: String,
    pub extended_message: Option<String>,
}

impl MassLynxError {
    pub fn new(error_code: i32, message: String) -> Self {
        Self { error_code, message, extended_message: None }
    }

    pub fn extended_new(error_code: i32, message: String, extended_message: Option<String>) -> Self {
        let mut this = Self::new(error_code, message);
        this.extended_message = extended_message;
        this
    }
}

impl Display for MassLynxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MassLynx Error occurred: ({}) {}",
            self.error_code, self.message
        )?;
        if let Some(s) = self.extended_message.as_ref() {
            write!(f, "; {s}")?;
        }
        Ok(())
    }
}

impl Error for MassLynxError {}

pub type MassLynxResult<T> = Result<T, MassLynxError>;

pub trait MassLynxReaderHelper {
    fn mass_lynx_error_for_code(error_code: i32) -> MassLynxError {
        let error_message = ptr::null();
        unsafe { ffi::getErrorMessage(error_code as c_int, &error_message) };
        let message = Self::to_string(error_message);
        MassLynxError {
            error_code,
            message,
            extended_message: None
        }
    }

    /// Assumes that the memory behind `c_string` is managed by the client or by the driver
    fn to_string(c_string: *const c_char) -> String {
        if c_string.is_null() {
            return String::new();
        } else {
            unsafe {
                let cs = CStr::from_ptr(c_string);
                let s = cs.to_string_lossy().to_string();
                return s;
            }
        }
    }

    /// Assumes that the memory behind `p_array` is managed by the client or by the driver
    fn to_vec<T: Copy>(p_array: *const T, n_size: c_int) -> Vec<T> {
        let mut buffer = Vec::new();
        Self::copy_data_into_vec(p_array, n_size, &mut buffer);
        buffer
    }

    /// Assumes that the memory behind `p_array` is managed by the client or by the driver
    fn copy_data_into_vec<T: Copy>(p_array: *const T, n_size: c_int, destination: &mut Vec<T>) {
        if p_array.is_null() {
            destination.clear();
            return;
        }
        if n_size < 1 {
            destination.clear();
            return;
        }

        destination.reserve_exact((n_size as usize).saturating_sub(destination.capacity()));
        for i in 0..n_size {
            destination.push(unsafe { *p_array.offset(i as isize) });
        }
    }

    fn free_memory(p_data: *const c_void) -> MassLynxResult<()> {
        fficall!({ ffi::releaseMemory(p_data) });
        Ok(())
    }
}

pub struct Helper();

impl MassLynxReaderHelper for Helper {}

pub fn get_mass_lynx_version() -> Option<String> {
    let mut buf = ptr::null();
    let code = unsafe { ffi::getVersionInfo(&mut buf) };
    if code != 0 {
        return None;
    }
    let s = Helper::to_string(buf);
    unsafe { ffi::releaseMemory(buf as *const c_void) };
    Some(s)
}

macro_rules! get_function_property {
    ($name:ident, $prop_type:ty, $ffi_fn:path) => {
        pub fn $name(&mut self, which_function: usize) -> MassLynxResult<$prop_type> {
            let mut prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe { ($ffi_fn)(self.0, which_function as c_int, &mut prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop)
            }
        }
    };

    ($name:ident, $prop_type:ty as bool, $ffi_fn:path) => {
        pub fn $name(&mut self, which_function: usize) -> MassLynxResult<bool> {
            let prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe { ($ffi_fn)(self.0, which_function as c_int, &prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop != 0)
            }
        }
    };

    ($name:ident, $prop_type:ty as $out_type:ty, $ffi_fn:path) => {
        pub fn $name(&mut self, which_function: usize) -> MassLynxResult<$out_type> {
            let mut prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe { ($ffi_fn)(self.0, which_function as c_int, &mut prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop as $out_type)
            }
        }
    };
}

macro_rules! get_function_property_two {
    ($name:ident, $prop_type1:ty, $prop_type2:ty, $ffi_fn:path) => {
        pub fn $name(
            &mut self,
            which_function: usize,
        ) -> MassLynxResult<($prop_type1, $prop_type2)> {
            let mut prop: $prop_type1 = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let mut prop2: $prop_type2 = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe { ($ffi_fn)(self.0, which_function as c_int, &mut prop, &mut prop2) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok((prop, prop2))
            }
        }
    };
}

macro_rules! get_file_reader_property {
    ($name:ident, $prop_type:ty, $ffi_fn:path) => {
        pub fn $name(&mut self) -> MassLynxResult<$prop_type> {
            let prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe { ($ffi_fn)(self.0, &prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop)
            }
        }
    };

    ($name:ident, $prop_type:ty as bool, $ffi_fn:path) => {
        pub fn $name(&mut self) -> MassLynxResult<bool> {
            let prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe { ($ffi_fn)(self.0, &prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop != 0)
            }
        }
    };
}

macro_rules! get_scan_property {
    ($name:ident, $prop_type:ty, $ffi_fn:path) => {
        pub fn $name(
            &mut self,
            which_function: usize,
            which_scan: usize,
        ) -> MassLynxResult<$prop_type> {
            let prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code =
                unsafe { ($ffi_fn)(self.0, which_function as c_int, which_scan as c_int, &prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop)
            }
        }
    };

    ($name:ident, $prop_type:ty as bool, $ffi_fn:path) => {
        pub fn $name(&mut self, which_function: usize, which_scan: usize) -> MassLynxResult<bool> {
            let prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code =
                unsafe { ($ffi_fn)(self.0, which_function as c_int, which_scan as c_int, &prop) };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop != 0)
            }
        }
    };

    ($name:ident, $prop_type:ty as $out_type:ty, $ffi_fn:path) => {
        pub fn $name(
            &mut self,
            which_function: usize,
            which_scan: usize,
        ) -> MassLynxResult<$out_type> {
            let mut prop: $prop_type = unsafe { mem::MaybeUninit::zeroed().assume_init() };
            let code = unsafe {
                ($ffi_fn)(
                    self.0,
                    which_function as c_int,
                    which_scan as c_int,
                    &mut prop,
                )
            };
            if code != 0 {
                Err(Self::mass_lynx_error_for_code(code))
            } else {
                Ok(prop as $out_type)
            }
        }
    };
}

pub struct MassLynxParameters(pub(crate) ffi::CMassLynxParameters);

impl MassLynxParameters {
    pub fn new() -> MassLynxResult<Self> {
        let this = Self(ptr::null_mut());
        let code = unsafe { ffi::createParameters(&this.0) };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(this)
        }
    }

    pub fn get<T: AsMassLynxItemKey>(&self, key: T) -> MassLynxResult<String> {
        let out = ptr::null();
        let code = unsafe { ffi::getParameterValue(self.0, key.as_key(), &out) };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(Self::to_string(out))
        }
    }

    pub fn set<T: AsMassLynxItemKey>(&mut self, key: T, value: String) -> MassLynxResult<()> {
        let value_ptr =
            CString::new(value).expect("Failed to convert value to C-compatible string");
        let code = unsafe { ffi::setParameterValue(self.0, key.as_key(), value_ptr.as_ptr()) };

        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(())
        }
    }

    pub fn get_raw_keys(&self) -> MassLynxResult<Vec<c_int>> {
        let keys = ptr::null();
        let size: c_int = 0;
        let code = unsafe { ffi::getParameterKeys(self.0, &keys, &size) };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(Self::to_vec(keys, size))
        }
    }

    pub fn get_keys<T: AsMassLynxItemKey>(&self) -> MassLynxResult<Vec<T>> {
        Ok(self
            .get_raw_keys()?
            .into_iter()
            .flat_map(|k| k.try_into())
            .collect())
    }

    pub fn iter_keys<T: AsMassLynxItemKey>(&self) -> impl Iterator<Item = T> {
        self.get_keys::<T>().into_iter().flatten()
    }

    pub fn iter<'a, T: AsMassLynxItemKey + 'a>(&'a self) -> impl Iterator<Item = (T, String)> + 'a {
        self.iter_keys().filter_map(|k| {
            let value = self.get(k);
            if let Ok(value) = value {
                Some((k, value))
            } else {
                None
            }
        })
    }

    pub fn to_hashmap<T: AsMassLynxItemKey + Eq + Hash>(&self) -> HashMap<T, String> {
        self.iter().collect()
    }

    pub const fn as_ptr_mut(&mut self) -> ffi::CMassLynxParameters {
        self.0
    }
}

impl MassLynxReaderHelper for MassLynxParameters {}

impl Drop for MassLynxParameters {
    fn drop(&mut self) {
        unsafe { ffi::destroyParameters(self.0) };
    }
}

macro_rules! impl_reader_apis {
    ($tp:ty, $base:expr) => {
        impl Default for $tp {
            fn default() -> Self {
                Self(ptr::null_mut())
            }
        }

        impl Drop for $tp {
            fn drop(&mut self) {
                trace!("Destroying Reader {:?}", Self::base_type());
                unsafe {
                    ffi::destroyRawReader(self.0);
                }
            }
        }

        impl MassLynxReaderHelper for $tp {}

        impl AsMassLynxSource for $tp {
            fn as_mass_lynx_source(&self) -> ffi::CMassLynxBaseReader {
                self.0
            }

            fn source_mut(&mut self) -> *mut ffi::CMassLynxBaseReader {
                &mut self.0
            }

            fn set_source(&mut self, source: ffi::CMassLynxBaseReader) {
                self.0 = source;
            }

            fn base_type() -> MassLynxBaseType {
                $base
            }
        }
    };
}

pub trait AsMassLynxSource: Default + MassLynxReaderHelper {
    fn as_mass_lynx_source(&self) -> ffi::CMassLynxBaseReader;

    fn source_mut(&mut self) -> *mut ffi::CMassLynxBaseReader;

    fn set_source(&mut self, source: ffi::CMassLynxBaseReader);

    fn base_type() -> MassLynxBaseType;

    fn from_path<P: AsRef<Path>>(path: P) -> MassLynxResult<Self> {
        let path = path.as_ref();
        let path_str = path.as_os_str();
        let s = path_str.as_encoded_bytes();
        // Ensure there's a trailing nul byte
        let s = CString::new(s).expect("Failed to convert path to a C-compatible string");
        let mut this = Self::default();
        fficall!({
            ffi::createRawReaderFromPath(s.as_ptr(), this.source_mut(), Self::base_type())
        });
        debug_assert!(!this.as_mass_lynx_source().is_null());
        Ok(this)
    }

    fn from_source<T: AsMassLynxSource>(source: &T) -> MassLynxResult<Self> {
        let mut this = Self::default();
        let reader_type = Self::base_type();
        let source_ptr = source.as_mass_lynx_source();
        fficall!({ ffi::createRawReaderFromReader(source_ptr, this.source_mut(), reader_type,) });
        debug_assert!(!this.as_mass_lynx_source().is_null());
        Ok(this)
    }
}

pub struct MassLynxInfoReader(ffi::CMassLynxBaseReader);

impl_reader_apis!(MassLynxInfoReader, MassLynxBaseType::INFO);

impl MassLynxInfoReader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MassLynxResult<Self> {
        <Self as AsMassLynxSource>::from_path(path)
    }

    pub fn function_count(&mut self) -> MassLynxResult<usize> {
        let count: c_uint = 0;
        let code = unsafe { ffi::getFunctionCount(self.0, &count) };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(count as usize)
        }
    }

    pub fn scan_count_for_function(&mut self, which_function: usize) -> MassLynxResult<usize> {
        let count: c_uint = 0;
        let code = unsafe { ffi::getScanCount(self.0, which_function as c_int, &count) };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(count as usize)
        }
    }

    get_file_reader_property!(is_lock_mass_corrected, i8 as bool, ffi::isLockMassCorrected);
    get_file_reader_property!(can_lock_mass_correct, i8 as bool, ffi::canLockMassCorrect);

    get_function_property!(
        get_function_type,
        MassLynxFunctionType,
        ffi::getFunctionType
    );
    get_function_property!(get_ion_mode, MassLynxIonMode, ffi::getIonMode);
    get_function_property!(is_continuum, i8 as bool, ffi::isContinuum);
    get_function_property!(
        get_drift_scan_count,
        c_uint as usize,
        ffi::getDriftScanCount
    );
    get_function_property!(get_mrm_count, c_int as usize, ffi::getMRMCount);

    get_function_property_two!(
        get_acquisition_time_range,
        c_float,
        c_float,
        ffi::getAcquisitionTimeRange
    );

    get_scan_property!(get_retention_time, c_float as f64, ffi::getRetentionTime);

    pub fn get_lock_mass_function(&self) -> MassLynxResult<(bool, usize)> {
        let mut has_lock_mass = 0;
        let mut lock_mass_function = 0;

        fficall!({ ffi::getLockMassFunction(self.0, &mut has_lock_mass, &mut lock_mass_function) });

        Ok((has_lock_mass != 0, lock_mass_function as usize))
    }

    pub fn get_drift_time(&mut self, which_drift: usize) -> MassLynxResult<f64> {
        let mut out = 0.0;

        fficall!({ ffi::getDriftTime(self.0, which_drift as c_int, &mut out) });

        Ok(out as f64)
    }

    pub fn get_ccs(&self, drift_time: f32, mz: f32, charge: i32) -> MassLynxResult<f32> {
        let mut out = 0.0;

        fficall!({
            ffi::getCollisionalCrossSection(self.0, drift_time, mz, charge, &mut out)
        });

        Ok(out)
    }

    pub fn get_drift_time_from_ccs(&self, ccs: f32, mz: f32, charge: i32) -> MassLynxResult<f32> {
        let mut out = 0.0;

        fficall!({
            ffi::getDriftTime_CCS(self.0, ccs, mz, charge, &mut out)
        });

        Ok(out)
    }

    pub fn get_acquisition_mass_range(&self, which_function: usize) -> MassLynxResult<(f64, f64)> {
        let low: c_float = 0.0;
        let high: c_float = 0.0;
        let code = unsafe {
            ffi::getAcquisitionMassRange(self.0, which_function as c_int, 0, &low, &high)
        };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok((low as f64, high as f64))
        }
    }

    pub fn get_header_items(
        &self,
        items: &[MassLynxHeaderItem],
    ) -> MassLynxResult<MassLynxParameters> {
        let params = MassLynxParameters::new()?;
        fficall!({
            ffi::getHeaderItemValue(self.0, items.as_ptr(), items.len() as c_int, params.0)
        });
        Ok(params)
    }

    pub fn get_acquisition_info(&mut self) -> MassLynxResult<MassLynxParameters> {
        let params = MassLynxParameters::new()?;
        fficall!({ ffi::getAcquisitionInfo(self.0, params.0) });
        Ok(params)
    }

    pub fn get_scan_items(&self, which_function: usize) -> MassLynxResult<MassLynxParameters> {
        let params = MassLynxParameters::new()?;

        fficall!({ ffi::getScanItemsInFunction(self.0, which_function as c_int, params.0) });

        Ok(params)
    }

    pub fn get_scan_item_values_for_scan(
        &self,
        which_function: usize,
        which_scan: usize,
        items: &[MassLynxScanItem],
    ) -> MassLynxResult<MassLynxParameters> {
        let params = MassLynxParameters::new()?;

        fficall!({
            ffi::getScanItemValue(
                self.0,
                which_function as c_int,
                which_scan as c_int,
                items.as_ptr(),
                items.len() as c_int,
                params.0,
            )
        });

        Ok(params)
    }
}

pub struct MassLynxScanReader(ffi::CMassLynxBaseReader);

impl_reader_apis!(MassLynxScanReader, MassLynxBaseType::SCAN);

impl MassLynxScanReader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MassLynxResult<Self> {
        <Self as AsMassLynxSource>::from_path(path)
    }

    pub fn read_scan_into(
        &mut self,
        which_function: usize,
        which_scan: usize,
        mz_array: &mut Vec<f32>,
        intensity_array: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let p_mzs = ptr::null();
        let p_intens = ptr::null();
        let size = 0;
        fficall!({
            ffi::readScan(
                self.0,
                which_function as c_int,
                which_scan as c_int,
                &p_mzs,
                &p_intens,
                &size,
            )
        });

        Self::copy_data_into_vec(p_mzs, size, mz_array);
        Self::copy_data_into_vec(p_intens, size, intensity_array);

        Ok(())
    }

    pub fn read_scan(
        &mut self,
        which_function: usize,
        which_scan: usize,
    ) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut mzs = Vec::new();
        let mut intens = Vec::new();
        self.read_scan_into(which_function, which_scan, &mut mzs, &mut intens)?;
        Ok((mzs, intens))
    }

    pub fn read_drift_scan_into(
        &mut self,
        which_function: usize,
        which_scan: usize,
        which_drift: usize,
        mz_array: &mut Vec<f32>,
        intensity_array: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let p_mzs = ptr::null();
        let p_intens = ptr::null();
        let size = 0;

        fficall!({
            ffi::readDriftScan(
                self.0,
                which_function as c_int,
                which_scan as c_int,
                which_drift as c_int,
                &p_mzs,
                &p_intens,
                &size,
            )
        });

        Self::copy_data_into_vec(p_mzs, size, mz_array);
        Self::copy_data_into_vec(p_intens, size, intensity_array);

        Ok(())
    }

    pub fn read_drift_scan(
        &mut self,
        which_function: usize,
        which_scan: usize,
        which_drift: usize,
    ) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let mut mzs = Vec::new();
        let mut intens = Vec::new();
        self.read_drift_scan_into(
            which_function,
            which_scan,
            which_drift,
            &mut mzs,
            &mut intens,
        )?;
        Ok((mzs, intens))
    }
}

pub struct MassLynxChromatogramReader(ffi::CMassLynxBaseReader);

impl_reader_apis!(MassLynxChromatogramReader, MassLynxBaseType::CHROM);

impl MassLynxChromatogramReader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MassLynxResult<Self> {
        <Self as AsMassLynxSource>::from_path(path)
    }

    pub fn read_tic_into(
        &mut self,
        which_function: usize,
        time_array: &mut Vec<f32>,
        intensity_array: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let p_times = ptr::null();
        let p_intens = ptr::null();
        let size = 0;
        fficall!({
            ffi::readTICChromatogram(self.0, which_function as c_int, &p_times, &p_intens, &size)
        });

        Self::copy_data_into_vec(p_times, size, time_array);
        Self::copy_data_into_vec(p_intens, size, intensity_array);

        Ok(())
    }

    pub fn read_bpi_into(
        &mut self,
        which_function: usize,
        time_array: &mut Vec<f32>,
        intensity_array: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let p_times = ptr::null();
        let p_intens = ptr::null();
        let size = 0;
        fficall!({
            ffi::readBPIChromatogram(self.0, which_function as c_int, &p_times, &p_intens, &size)
        });

        Self::copy_data_into_vec(p_times, size, time_array);
        Self::copy_data_into_vec(p_intens, size, intensity_array);
        Ok(())
    }

    pub fn read_mass_chromatograms_into(
        &mut self,
        which_function: usize,
        mass_list: &[f32],
        time_array: &mut Vec<f32>,
        intensity_arrays: &mut [Vec<f32>],
        mass_window: f32,
        daughters: bool,
    ) -> MassLynxResult<()> {
        let p_times = ptr::null();
        let p_intens = ptr::null();
        let size = 0;

        fficall!({
            ffi::readMassChromatograms(
                self.0,
                which_function as c_int,
                mass_list.as_ptr(),
                mass_list.len() as c_int,
                &p_times,
                &p_intens,
                mass_window,
                daughters as i8,
                &size,
            )
        });

        Self::copy_data_into_vec(p_times, size, time_array);

        for (i, buf) in intensity_arrays.iter_mut().enumerate() {
            let offset_p_intens = unsafe { p_intens.offset(size as isize * i as isize) };
            Self::copy_data_into_vec(offset_p_intens, size, buf);
        }
        Self::free_memory(p_times as *const c_void)?;
        Self::free_memory(p_intens as *const c_void)?;
        Ok(())
    }

    pub fn read_mass_chromatogram_into(
        &mut self,
        which_function: usize,
        mass: f32,
        time_array: &mut Vec<f32>,
        intensity_array: &mut Vec<f32>,
        mass_window: f32,
        daughters: bool,
    ) -> MassLynxResult<()> {
        let p_times = ptr::null();
        let p_intens = ptr::null();
        let size = 0;

        fficall!({
            ffi::readMassChromatograms(
                self.0,
                which_function as c_int,
                [mass].as_ptr(),
                1,
                &p_times,
                &p_intens,
                mass_window,
                daughters as i8,
                &size,
            )
        });

        Self::copy_data_into_vec(p_times, size, time_array);
        Self::copy_data_into_vec(p_intens, size, intensity_array);
        Self::free_memory(p_times as *const c_void)?;
        Self::free_memory(p_intens as *const c_void)?;
        Ok(())
    }

    pub fn read_mobilogram_into(
        &mut self,
        which_function: usize,
        start_scan: usize,
        end_scan: usize,
        start_mass: f32,
        end_mass: f32,
        drift_bins: &mut Vec<i32>,
        intensity_array: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let p_drifts = ptr::null();
        let p_intens = ptr::null();
        let size = 0;

        fficall!({
            ffi::readMobillogram(
                self.0,
                which_function as c_int,
                start_scan as c_int,
                end_scan as c_int,
                start_mass,
                end_mass,
                &p_drifts,
                &p_intens,
                &size,
            )
        });

        Self::copy_data_into_vec(p_drifts, size, drift_bins);
        Self::copy_data_into_vec(p_intens, size, intensity_array);
        Self::free_memory(p_drifts as *const c_void)?;
        Self::free_memory(p_intens as *const c_void)?;

        Ok(())
    }
}

pub struct MassLynxLockMassProcessor(ffi::CMassLynxBaseProcessor);

impl MassLynxLockMassProcessor {
    pub fn new() -> MassLynxResult<Self> {
        let this = Self::default();
        let code = unsafe {
            ffi::createRawProcessor(
                &this.0,
                MassLynxBaseType::LOCKMASS,
                None,
                ptr::addr_of!(this) as *const c_void,
            )
        };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(this)
        }
    }

    pub fn set_raw_data_from_reader<T: AsMassLynxSource>(
        &mut self,
        raw_reader: &T,
    ) -> MassLynxResult<()> {
        fficall!({ ffi::setRawReader(self.0, raw_reader.as_mass_lynx_source()) });

        Ok(())
    }

    pub fn set_raw_data_from_path(&mut self, path: String) -> MassLynxResult<()> {
        let cpath = CString::new(path).expect("Failed to convert path to C-compatible string");
        fficall!({ ffi::setRawPath(self.0, cpath.as_ptr() as *const i8) });
        Ok(())
    }

    pub fn is_lock_mass_corrected(&self) -> MassLynxResult<bool> {
        let is_corrected = 0;
        fficall!({ ffi::LMP_isLockMassCorrected(self.0, &is_corrected) });
        Ok(is_corrected != 0)
    }

    pub fn can_lock_mass_correct(&self) -> MassLynxResult<bool> {
        let can_correct = 0;
        fficall!({ ffi::LMP_canLockMassCorrect(self.0, &can_correct) });

        Ok(can_correct != 0)
    }

    pub fn remove_lock_mass_correction(&mut self) -> MassLynxResult<()> {
        fficall!({ ffi::removeLockMassCorrection(self.0) });
        Ok(())
    }

    pub fn get_lock_mass_correction(&self, retention_time: f32) -> MassLynxResult<f32> {
        let gain = 0.0;
        fficall!({ ffi::getLockMassCorrection(self.0, retention_time, &gain) });

        Ok(gain)
    }

    pub fn set_parameters(&mut self, params: &MassLynxParameters) -> MassLynxResult<()> {
        fficall!({ ffi::setLockMassParameters(self.0, params.0) });
        Ok(())
    }

    pub fn lock_mass_correct(&mut self) -> MassLynxResult<bool> {
        let corrected = 0;
        fficall!({ ffi::lockMassCorrect(self.0, &corrected) });
        Ok(corrected != 0)
    }

    pub fn auto_lock_mass_correct(&mut self, force: bool) -> MassLynxResult<bool> {
        let mut corrected = 0;
        fficall!({
            ffi::autoLockMassCorrect(self.0, force as c_char, &mut corrected)
        });
        Ok(corrected != 0)
    }

    pub fn get_candidates(
        &mut self,
        masses: &mut Vec<f32>,
        intensities: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let mzs = ptr::null();
        let intens = ptr::null();
        let size = 0;

        fficall!({ ffi::getLockMassCandidates(self.0, &mzs, &intens, &size) });

        Self::copy_data_into_vec(mzs, size, masses);
        fficall!({ ffi::releaseMemory(mzs as *const c_void) });
        Self::copy_data_into_vec(intens, size, intensities);
        fficall!({ ffi::releaseMemory(intens as *const c_void) });
        Ok(())
    }
}

impl MassLynxReaderHelper for MassLynxLockMassProcessor {}

impl Drop for MassLynxLockMassProcessor {
    fn drop(&mut self) {
        unsafe {
            ffi::destroyRawProcessor(self.0);
        }
    }
}

impl Default for MassLynxLockMassProcessor {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

pub struct MassLynxAnalogReader(ffi::CMassLynxBaseReader);

impl MassLynxAnalogReader {
    pub fn from_path<P: AsRef<Path>>(path: P) -> MassLynxResult<Self> {
        <Self as AsMassLynxSource>::from_path(path)
    }

    pub fn channel_count(&self) -> MassLynxResult<usize> {
        let mut size = 0;

        fficall!({ ffi::getChannelCount(self.0, &mut size) });

        Ok(size as usize)
    }

    pub fn read_channel(&mut self, which_channel: usize) -> MassLynxResult<(Vec<f32>, Vec<f32>)> {
        let times = ptr::null();
        let ints = ptr::null();
        let mut size = 0;
        fficall!({ ffi::readChannel(self.0, which_channel as c_int, &times, &ints, &mut size) });

        let mut times_: Vec<f32> = Vec::new();
        let mut ints_: Vec<f32> = Vec::new();

        Self::copy_data_into_vec(times, size, &mut times_);
        Self::copy_data_into_vec(ints, size, &mut ints_);

        Ok((times_, ints_))
    }

    pub fn channel_description(&mut self, which_channel: usize) -> MassLynxResult<String> {
        let s = ptr::null();

        fficall!({ ffi::getChannelDesciption(self.0, which_channel as c_int, &s) });

        Ok(Self::to_string(s))
    }

    pub fn channel_units(&mut self, which_channel: usize) -> MassLynxResult<String> {
        let s = ptr::null();

        fficall!({ ffi::getChannelUnits(self.0, which_channel as c_int, &s) });

        Ok(Self::to_string(s))
    }
}

impl_reader_apis!(MassLynxAnalogReader, MassLynxBaseType::ANALOG);

pub struct MassLynxScanProcessor(ffi::CMassLynxBaseProcessor);

impl MassLynxScanProcessor {
    pub fn new() -> MassLynxResult<Self> {
        let this = Self::default();
        let code = unsafe {
            ffi::createRawProcessor(
                &this.0,
                MassLynxBaseType::SCAN,
                None,
                ptr::addr_of!(this) as *const c_void,
            )
        };
        if code != 0 {
            Err(Self::mass_lynx_error_for_code(code))
        } else {
            Ok(this)
        }
    }

    pub fn set_raw_data_from_reader<T: AsMassLynxSource>(
        &mut self,
        raw_reader: &T,
    ) -> MassLynxResult<()> {
        fficall!({ ffi::setRawReader(self.0, raw_reader.as_mass_lynx_source()) });

        Ok(())
    }

    pub fn set_raw_data_from_path(&mut self, path: String) -> MassLynxResult<()> {
        let cpath = CString::new(path).expect("Failed to convert path to C-compatible string");
        fficall!({ ffi::setRawPath(self.0, cpath.as_ptr() as *const i8) });
        Ok(())
    }

    pub fn load(&mut self, which_function: usize, which_scan: usize) -> MassLynxResult<()> {
        fficall!({
            ffi::combineScan(
                self.0,
                which_function as c_int,
                which_scan as c_int,
                which_scan as c_int,
            )
        });
        Ok(())
    }

    pub fn load_drift(
        &mut self,
        which_function: usize,
        which_scan: usize,
        which_drift: usize,
    ) -> MassLynxResult<()> {
        fficall!({
            ffi::combineDriftScan(
                self.0,
                which_function as c_int,
                which_scan as c_int,
                which_scan as c_int,
                which_drift as c_int,
                which_drift as c_int,
            )
        });
        Ok(())
    }

    pub fn combine(
        &mut self,
        which_function: usize,
        start_scan: usize,
        end_scan: usize,
    ) -> MassLynxResult<()> {
        fficall!({
            ffi::combineScan(
                self.0,
                which_function as c_int,
                start_scan as c_int,
                end_scan as c_int,
            )
        });
        Ok(())
    }

    pub fn combine_drift(
        &mut self,
        which_function: usize,
        start_scan: usize,
        end_scan: usize,
        start_drift: usize,
        end_drift: usize,
    ) -> MassLynxResult<()> {
        fficall!({
            ffi::combineDriftScan(
                self.0,
                which_function as c_int,
                start_scan as c_int,
                end_scan as c_int,
                start_drift as c_int,
                end_drift as c_int,
            )
        });
        Ok(())
    }

    pub fn set_centroid_parameters(&mut self, params: MassLynxParameters) -> MassLynxResult<()> {
        fficall!({ ffi::setCentroidParameter(self.0, params.0) });
        Ok(())
    }

    pub fn set_smooth_parameters(&mut self, params: MassLynxParameters) -> MassLynxResult<()> {
        fficall!({ ffi::setSmoothParameter(self.0, params.0) });
        Ok(())
    }

    pub fn set_scan(&mut self, mz_array: &[f32], intensity_array: &[f32]) -> MassLynxResult<()> {
        fficall!({
            ffi::setScan(
                self.0,
                mz_array.as_ptr(),
                intensity_array.as_ptr(),
                mz_array.len() as c_int,
                intensity_array.len() as c_int,
            )
        });
        Ok(())
    }

    pub fn centroid(&mut self) -> MassLynxResult<()> {
        fficall!({ ffi::centroidScan(self.0) });
        Ok(())
    }

    pub fn smooth(&mut self) -> MassLynxResult<()> {
        fficall!({ ffi::smoothScan(self.0) });
        Ok(())
    }

    pub fn get(
        &self,
        mz_array: &mut Vec<f32>,
        intensity_array: &mut Vec<f32>,
    ) -> MassLynxResult<()> {
        let mzs = ptr::null();
        let intens = ptr::null();
        let mut size = 0;
        fficall!({ ffi::getScan(self.0, &mzs, &intens, &mut size) });

        Self::copy_data_into_vec(mzs, size, mz_array);
        Self::copy_data_into_vec(intens, size, intensity_array);
        Ok(())
    }
}

impl MassLynxReaderHelper for MassLynxScanProcessor {}

impl Drop for MassLynxScanProcessor {
    fn drop(&mut self) {
        unsafe {
            ffi::destroyRawProcessor(self.0);
        }
    }
}

impl Default for MassLynxScanProcessor {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}
