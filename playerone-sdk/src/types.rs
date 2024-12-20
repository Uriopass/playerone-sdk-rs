use std::ffi::CStr;
use std::fmt::{Display, Formatter};

use playerone_sdk_sys::{
    _POABayerPattern, _POACameraProperties, _POAConfig, _POAImgFormat, _POAValueType, POABool,
    POAConfig, POAConfigAttributes, POAErrors, POAImgFormat,
};

#[derive(Debug, Clone)]
pub struct CameraProperties {
    /// the camera name
    pub camera_model_name: String,
    /// user custom name, it will be added after the camera name, max len 16 bytes,like:Mars-C [Juno], default is empty
    pub user_custom_id: String,
    pub camera_id: u32,
    /// max width of the camera
    pub max_width: u32,
    /// max height of the camera
    pub max_height: u32,
    /// ADC depth of CMOS sensor
    pub bit_depth: u32,
    /// is a color camera or not
    pub is_color_camera: bool,
    /// does the camera have ST4 port, if not, camera don't support ST4 guide
    pub is_has_st_4_port: bool,
    /// does the camera have cooler assembly, generally, the cooled camera with cooler, window heater and fan
    pub is_has_cooler: bool,
    /// is usb3.0 speed connection
    pub is_usb_3_speed: bool,
    /// the bayer filter pattern of camera
    pub bayer_pattern: BayerPattern,
    /// camera pixel size(unit: um)
    pub pixel_size: f64,
    /// the serial number of camera, unique
    pub serial_number: String,
    /// the sensor model name, eg: IMX462
    pub sensor_model_name: String,
    /// the path of the camera in the computer host
    pub local_path: String,
    /// bins supported by the camera, 1 == bin1, 2 == bin2,...
    pub bins: Vec<u32>,
    /// image data format supported by the camera
    pub img_formats: Vec<POAImgFormat>,
    /// does the camera sensor support hardware bin
    pub is_support_hard_bin: bool,
    /// camera's Product ID, note: the vID of PlayerOne is 0xA0A0
    pub product_id: i32,
}

impl From<_POACameraProperties> for CameraProperties {
    fn from(value: _POACameraProperties) -> Self {
        let camera_model_name = unsafe {
            CStr::from_ptr(value.cameraModelName.as_ptr())
                .to_string_lossy()
                .to_string()
        };
        let user_custom_id = unsafe {
            CStr::from_ptr(value.userCustomID.as_ptr())
                .to_string_lossy()
                .to_string()
        };
        let sn = unsafe {
            CStr::from_ptr(value.SN.as_ptr())
                .to_string_lossy()
                .to_string()
        };
        let sensor_model_name = unsafe {
            CStr::from_ptr(value.sensorModelName.as_ptr())
                .to_string_lossy()
                .to_string()
        };
        let local_path = unsafe {
            CStr::from_ptr(value.localPath.as_ptr())
                .to_string_lossy()
                .to_string()
        };

        let mut bins = Vec::with_capacity(value.bins.len());
        for bin in value.bins {
            if bin <= 0 {
                break;
            }
            bins.push(bin as u32);
        }

        let mut img_formats = Vec::with_capacity(value.imgFormats.len());
        for img_format in value.imgFormats {
            if img_format == _POAImgFormat::POA_END {
                break;
            }
            img_formats.push(img_format.into());
        }

        Self {
            camera_model_name,
            user_custom_id,
            camera_id: value.cameraID as u32,
            max_width: value.maxWidth as u32,
            max_height: value.maxHeight as u32,
            bit_depth: value.bitDepth as u32,
            is_color_camera: value.isColorCamera == POABool::POA_TRUE,
            is_has_st_4_port: value.isHasST4Port == POABool::POA_TRUE,
            is_has_cooler: value.isHasCooler == POABool::POA_TRUE,
            is_usb_3_speed: value.isUSB3Speed == POABool::POA_TRUE,
            bayer_pattern: value.bayerPattern.into(),
            pixel_size: value.pixelSize,
            serial_number: sn,
            sensor_model_name,
            local_path,
            bins,
            img_formats,
            is_support_hard_bin: value.isSupportHardBin == POABool::POA_TRUE,
            product_id: value.pID.try_into().expect("c_int is not i32"),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ImageFormat {
    /// 8bit raw data, 1 pixel 1 byte, value range[0, 255]
    RAW8,
    /// 16bit raw data, 1 pixel 2 bytes, value range[0, 65535]
    RAW16,
    /// RGB888 color data, 1 pixel 3 bytes, value range[0, 255] (only color camera)
    RGB24,
    /// 8bit monochrome data, convert the Bayer Filter Array to monochrome data. 1 pixel 1 byte, value range[0, 255] (only color camera)
    MONO8,
}

impl ImageFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        use ImageFormat::*;
        match self {
            RAW8 => 1,
            RAW16 => 2,
            RGB24 => 3,
            MONO8 => 1,
        }
    }
}

impl From<_POAImgFormat> for ImageFormat {
    fn from(value: _POAImgFormat) -> Self {
        use ImageFormat::*;
        use _POAImgFormat::*;
        match value {
            POA_RAW8 => RAW8,
            POA_RAW16 => RAW16,
            POA_RGB24 => RGB24,
            POA_MONO8 => MONO8,
            POA_END => unreachable!("POA_END should have been parsed before"),
        }
    }
}

impl Into<_POAImgFormat> for ImageFormat {
    fn into(self) -> _POAImgFormat {
        use ImageFormat::*;
        use _POAImgFormat::*;
        match self {
            RAW8 => POA_RAW8,
            RAW16 => POA_RAW16,
            RGB24 => POA_RGB24,
            MONO8 => POA_MONO8,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BayerPattern {
    /// Monochrome, the mono camera with this
    MONO,
    /// RGGB
    RG,
    /// BGGR
    BG,
    /// GRBG
    GR,
    /// GBRG
    GB,
}

impl From<_POABayerPattern> for BayerPattern {
    fn from(value: _POABayerPattern) -> Self {
        use BayerPattern::*;
        use _POABayerPattern::*;
        match value {
            POA_BAYER_RG => RG,
            POA_BAYER_BG => BG,
            POA_BAYER_GR => GR,
            POA_BAYER_GB => GB,
            POA_BAYER_MONO => MONO,
        }
    }
}

#[derive(Debug)]
pub struct ConfigBounds<T> {
    pub min: T,
    pub max: T,
    pub default: T,
    pub conf_name: String,
    pub description: String,
}

trait FromAttribute: Sized {
    fn from_attribute(value: POAConfigAttributes) -> (Self, Self, Self);
}

impl FromAttribute for i64 {
    fn from_attribute(value: POAConfigAttributes) -> (Self, Self, Self) {
        if value.valueType != _POAValueType::VAL_INT {
            let name = unsafe {
                CStr::from_ptr(value.szConfName.as_ptr())
                    .to_string_lossy()
                    .to_string()
            };
            panic!("valueType is not VAL_INT for {}", name);
        }
        unsafe {
            (
                value.minValue.intValue as i64,
                value.maxValue.intValue as i64,
                value.defaultValue.intValue as i64,
            )
        }
    }
}

impl FromAttribute for f64 {
    fn from_attribute(value: POAConfigAttributes) -> (Self, Self, Self) {
        if value.valueType != _POAValueType::VAL_FLOAT {
            let name = unsafe {
                CStr::from_ptr(value.szConfName.as_ptr())
                    .to_string_lossy()
                    .to_string()
            };
            panic!("valueType is not VAL_FLOAT for {}", name);
        }
        unsafe {
            (
                value.minValue.floatValue,
                value.maxValue.floatValue,
                value.defaultValue.floatValue,
            )
        }
    }
}

impl FromAttribute for bool {
    fn from_attribute(value: POAConfigAttributes) -> (Self, Self, Self) {
        if value.valueType != _POAValueType::VAL_BOOL {
            let name = unsafe {
                CStr::from_ptr(value.szConfName.as_ptr())
                    .to_string_lossy()
                    .to_string()
            };
            panic!("valueType is not VAL_BOOL for {}", name);
        }
        unsafe {
            (
                value.minValue.boolValue == POABool::POA_TRUE,
                value.maxValue.boolValue == POABool::POA_TRUE,
                value.defaultValue.boolValue == POABool::POA_TRUE,
            )
        }
    }
}

impl<T: FromAttribute> From<POAConfigAttributes> for ConfigBounds<T> {
    fn from(value: POAConfigAttributes) -> Self {
        let (min, max, default) = T::from_attribute(value);

        Self {
            min,
            max,
            default,
            conf_name: unsafe {
                CStr::from_ptr(value.szConfName.as_ptr())
                    .to_string_lossy()
                    .to_string()
            },
            description: unsafe {
                CStr::from_ptr(value.szDescription.as_ptr())
                    .to_string_lossy()
                    .to_string()
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ConfigKind {
    /// exposure time(unit: us), read-write
    Exposure,
    /// gain, read-write
    Gain,
    /// hardware bin, read-write
    HardwareBin,
    /// camera temperature(uint: C), read-only
    Temperature,
    /// red pixels coefficient of white balance, read-write
    WbR,
    /// green pixels coefficient of white balance, read-write
    WbG,
    /// blue pixels coefficient of white balance, read-write
    WbB,
    /// camera offset, read-write
    Offset,
    /// maximum gain when auto-adjust, read-write
    AutoexpoMaxGain,
    /// maximum exposure when auto-adjust(uint: ms), read-write
    AutoexpoMaxExposure,
    /// target brightness when auto-adjust, read-write
    AutoexpoBrightness,
    /// ST4 guide north, generally,it's DEC+ on the mount, read-write
    GuideNorth,
    /// ST4 guide south, generally,it's DEC- on the mount, read-write
    GuideSouth,
    /// ST4 guide east, generally,it's RA+ on the mount, read-write
    GuideEast,
    /// ST4 guide west, generally,it's RA- on the mount, read-write
    GuideWest,
    /// e/ADU, This value will change with gain, read-only
    Egain,
    /// cooler power percentage[0-100%](only cool camera), read-only
    CoolerPower,
    /// camera target temperature(uint: C), read-write
    TargetTemp,
    /// turn cooler(and fan) on or off, read-write
    Cooler,
    /// (deprecated)get state of lens heater(on or off), read-only
    Heater,
    /// lens heater power percentage[0-100%], read-write
    HeaterPower,
    /// radiator fan power percentage[0-100%], read-write
    FanPower,
    /// no flip, Note: set this config(POASetConfig), the 'confValue' will be ignored, read-write
    FlipNone,
    /// flip the image horizontally, Note: set this config(POASetConfig), the 'confValue' will be ignored, read-write
    FlipHori,
    /// flip the image vertically, Note: set this config(POASetConfig), the 'confValue' will be ignored, read-write
    FlipVert,
    /// flip the image horizontally and vertically, Note: set this config(POASetConfig), the 'confValue' will be ignored, read-write
    FlipBoth,
    /// Frame rate limit, the range:[0, 2000], 0 means no limit, read-write
    FrameLimit,
    /// High Quality Image, for those without DDR camera(guide camera), if set POA_TRUE, this will reduce the waviness and stripe of the image,\n< but frame rate may go down, note: this config has no effect on those cameras that with DDR. read-write
    Hqi,
    /// USB bandwidth limit, read-write
    UsbBandwidthLimit,
    /// take the sum of pixels after binning, POA_TRUE is sum and POA_FLASE is average, default is POA_FLASE, read-write
    PixelBinSum,
    /// only for color camera, when set to POA_TRUE, pixel binning will use neighbour pixels and image after binning will lose the bayer pattern, read-write
    MonoBin,
}

#[derive(Debug)]
pub struct AllConfigBounds {
    /// exposure time(unit: us)
    pub exposure: ConfigBounds<i64>,
    pub gain: ConfigBounds<i64>,
    /// red pixels coefficient of white balance
    pub wb_r: Option<ConfigBounds<i64>>,
    /// green pixels coefficient of white balance
    pub wb_g: Option<ConfigBounds<i64>>,
    /// blue pixels coefficient of white balance
    pub wb_b: Option<ConfigBounds<i64>>,
    /// gain offset (meaning: what 0 represents)
    pub offset: ConfigBounds<i64>,
    /// maximum gain when auto-adjust
    pub auto_max_gain: ConfigBounds<i64>,
    /// maximum exposure when auto-adjust(unit: ms)
    pub auto_max_exposure: ConfigBounds<i64>,
    /// target brightness when auto-adjust
    pub auto_target_brightness: ConfigBounds<i64>,
    /// frame rate limit, the range:[0, 2000], 0 means no limit
    pub frame_limit: ConfigBounds<i64>,
    /// USB bandwidth limit [0, 100]%, default is 90
    pub usb_bandwidth_limit: ConfigBounds<i64>,
    /// cooler power percentage[0-100%]
    pub cooler_power: Option<ConfigBounds<i64>>,
    /// camera target temperature (Celsius)
    pub target_temperature: Option<ConfigBounds<i64>>,
    /// heater power percentage[0-100%]
    pub heater_power: Option<ConfigBounds<i64>>,
    /// radiator fan power percentage[0-100%]
    pub fan_power: Option<ConfigBounds<i64>>,
}

impl From<Vec<POAConfigAttributes>> for AllConfigBounds {
    fn from(values: Vec<POAConfigAttributes>) -> Self {
        let mut exposure: Option<ConfigBounds<i64>> = None;
        let mut gain: Option<ConfigBounds<i64>> = None;
        let mut wb_r: Option<ConfigBounds<i64>> = None;
        let mut wb_g: Option<ConfigBounds<i64>> = None;
        let mut wb_b: Option<ConfigBounds<i64>> = None;
        let mut offset: Option<ConfigBounds<i64>> = None;
        let mut autoexpo_max_gain: Option<ConfigBounds<i64>> = None;
        let mut autoexpo_max_exposure: Option<ConfigBounds<i64>> = None;
        let mut autoexpo_brightness: Option<ConfigBounds<i64>> = None;
        let mut cooler_power: Option<ConfigBounds<i64>> = None;
        let mut target_temp: Option<ConfigBounds<i64>> = None;
        let mut heater_power: Option<ConfigBounds<i64>> = None;
        let mut fan_power: Option<ConfigBounds<i64>> = None;
        let mut frame_limit: Option<ConfigBounds<i64>> = None;
        let mut usb_bandwidth_limit: Option<ConfigBounds<i64>> = None;

        for value in values {
            let kind = value.configID.into();
            match kind {
                ConfigKind::Exposure => {
                    exposure = Some(ConfigBounds::from(value));
                }
                ConfigKind::Gain => {
                    gain = Some(ConfigBounds::from(value));
                }
                ConfigKind::WbR => {
                    wb_r = Some(ConfigBounds::from(value));
                }
                ConfigKind::WbG => {
                    wb_g = Some(ConfigBounds::from(value));
                }
                ConfigKind::WbB => {
                    wb_b = Some(ConfigBounds::from(value));
                }
                ConfigKind::Offset => {
                    offset = Some(ConfigBounds::from(value));
                }
                ConfigKind::AutoexpoMaxGain => {
                    autoexpo_max_gain = Some(ConfigBounds::from(value));
                }
                ConfigKind::AutoexpoMaxExposure => {
                    autoexpo_max_exposure = Some(ConfigBounds::from(value));
                }
                ConfigKind::AutoexpoBrightness => {
                    autoexpo_brightness = Some(ConfigBounds::from(value));
                }
                ConfigKind::CoolerPower => {
                    cooler_power = Some(ConfigBounds::from(value));
                }
                ConfigKind::TargetTemp => {
                    target_temp = Some(ConfigBounds::from(value));
                }
                ConfigKind::HeaterPower => {
                    heater_power = Some(ConfigBounds::from(value));
                }
                ConfigKind::FanPower => {
                    fan_power = Some(ConfigBounds::from(value));
                }
                ConfigKind::FrameLimit => {
                    frame_limit = Some(ConfigBounds::from(value));
                }
                ConfigKind::UsbBandwidthLimit => {
                    usb_bandwidth_limit = Some(ConfigBounds::from(value));
                }
                _ => {}
            }
        }

        Self {
            exposure: exposure.expect("exposure is not found"),
            gain: gain.expect("gain is not found"),
            wb_r,
            wb_g,
            wb_b,
            offset: offset.expect("offset is not found"),
            auto_max_gain: autoexpo_max_gain.expect("autoexpo_max_gain is not found"),
            auto_max_exposure: autoexpo_max_exposure.expect("autoexpo_max_exposure is not found"),
            auto_target_brightness: autoexpo_brightness.expect("autoexpo_brightness is not found"),
            cooler_power,
            target_temperature: target_temp,
            heater_power,
            fan_power,
            frame_limit: frame_limit.expect("frame_limit is not found"),
            usb_bandwidth_limit: usb_bandwidth_limit.expect("usb_bandwidth_limit is not found"),
        }
    }
}

impl From<POAConfig> for ConfigKind {
    fn from(value: POAConfig) -> Self {
        use ConfigKind::*;
        use _POAConfig::*;
        match value {
            POA_EXPOSURE => Exposure,
            POA_GAIN => Gain,
            POA_HARDWARE_BIN => HardwareBin,
            POA_TEMPERATURE => Temperature,
            POA_WB_R => WbR,
            POA_WB_G => WbG,
            POA_WB_B => WbB,
            POA_OFFSET => Offset,
            POA_AUTOEXPO_MAX_GAIN => AutoexpoMaxGain,
            POA_AUTOEXPO_MAX_EXPOSURE => AutoexpoMaxExposure,
            POA_AUTOEXPO_BRIGHTNESS => AutoexpoBrightness,
            POA_GUIDE_NORTH => GuideNorth,
            POA_GUIDE_SOUTH => GuideSouth,
            POA_GUIDE_EAST => GuideEast,
            POA_GUIDE_WEST => GuideWest,
            POA_EGAIN => Egain,
            POA_COOLER_POWER => CoolerPower,
            POA_TARGET_TEMP => TargetTemp,
            POA_COOLER => Cooler,
            POA_HEATER => Heater,
            POA_HEATER_POWER => HeaterPower,
            POA_FAN_POWER => FanPower,
            POA_FLIP_NONE => FlipNone,
            POA_FLIP_HORI => FlipHori,
            POA_FLIP_VERT => FlipVert,
            POA_FLIP_BOTH => FlipBoth,
            POA_FRAME_LIMIT => FrameLimit,
            POA_HQI => Hqi,
            POA_USB_BANDWIDTH_LIMIT => UsbBandwidthLimit,
            POA_PIXEL_BIN_SUM => PixelBinSum,
            POA_MONO_BIN => MonoBin,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Error {
    /// invalid index, means the index is < 0 or >= the count( camera or config)
    InvalidIndex,
    InvalidCameraId,
    InvalidConfig,
    InvalidArgument,
    NotOpened,
    DeviceNotFound,
    OutOfBounds,
    ExposureFailed,
    Timeout,
    BufferSizeTooSmall,
    /// camera is exposing. must stop exposure first
    Exposing,
    NullPointer,
    ConfigNotWritable,
    ConfigNotReadable,
    AccessDenied,
    /// maybe the camera disconnected suddenly
    OperationFailed,
    MemoryAllocationFailed,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        write!(
            f,
            "{}",
            match self {
                InvalidIndex => "invalid index",
                InvalidCameraId => "invalid camera id",
                InvalidConfig => "invalid config",
                InvalidArgument => "invalid argument",
                NotOpened => "camera is not opened",
                DeviceNotFound => "device not found",
                OutOfBounds => "out of bounds",
                ExposureFailed => "exposure failed",
                Timeout => "timeout",
                BufferSizeTooSmall => "buffer size too small",
                Exposing => "camera is exposing",
                NullPointer => "null pointer",
                ConfigNotWritable => "config is not writable",
                ConfigNotReadable => "config is not readable",
                AccessDenied => "access denied",
                OperationFailed => "operation failed",
                MemoryAllocationFailed => "memory allocation failed",
            }
        )
    }
}

impl std::error::Error for Error {}

impl From<POAErrors> for Error {
    fn from(value: POAErrors) -> Self {
        use Error::*;
        use POAErrors::*;
        match value {
            POA_OK => unreachable!("POA_OK should have been checked before"),
            POA_ERROR_INVALID_INDEX => InvalidIndex,
            POA_ERROR_INVALID_ID => InvalidCameraId,
            POA_ERROR_INVALID_CONFIG => InvalidConfig,
            POA_ERROR_INVALID_ARGU => InvalidArgument,
            POA_ERROR_NOT_OPENED => NotOpened,
            POA_ERROR_DEVICE_NOT_FOUND => DeviceNotFound,
            POA_ERROR_OUT_OF_LIMIT => OutOfBounds,
            POA_ERROR_EXPOSURE_FAILED => ExposureFailed,
            POA_ERROR_TIMEOUT => Timeout,
            POA_ERROR_SIZE_LESS => BufferSizeTooSmall,
            POA_ERROR_EXPOSING => Exposing,
            POA_ERROR_POINTER => NullPointer,
            POA_ERROR_CONF_CANNOT_WRITE => ConfigNotWritable,
            POA_ERROR_CONF_CANNOT_READ => ConfigNotReadable,
            POA_ERROR_ACCESS_DENIED => AccessDenied,
            POA_ERROR_OPERATION_FAILED => OperationFailed,
            POA_ERROR_MEMORY_FAILED => MemoryAllocationFailed,
        }
    }
}
