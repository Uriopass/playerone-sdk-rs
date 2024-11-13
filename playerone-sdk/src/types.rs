use std::ffi::CStr;

use playerone_sdk_sys::{
    _POABayerPattern, _POACameraProperties, _POAConfig, _POAImgFormat, _POAValueType, POABool,
    POAConfig, POAConfigAttributes, POAImgFormat,
};

#[derive(Debug, Clone)]
pub struct CameraProperties {
    /// the camera name
    pub camera_model_name: String,
    /// user custom name, it will be added after the camera name, max len 16 bytes,like:Mars-C [Juno], default is empty
    pub user_custom_id: String,
    /// it's unique,camera can be controlled and set by the cameraID
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
    /// the serial number of camera,it's unique
    pub serial_number: String,
    /// the sersor model(name) of camera, eg: IMX462
    pub sensor_model_name: String,
    /// the path of the camera in the computer host
    pub local_path: String,
    /// bins supported by the camera, 1 == bin1, 2 == bin2,...
    pub bins: Vec<u32>,
    /// image data format supported by the camera
    pub img_formats: Vec<POAImgFormat>,
    /// does the camera sensor support hardware bin (since V3.3.0)
    pub is_support_hard_bin: bool,
    /// camera's Product ID, note: the vID of PlayerOne is 0xA0A0 (since V3.3.0)
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
            if bin == -1 {
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
pub enum ImgFormat {
    /// 8bit raw data, 1 pixel 1 byte, value range[0, 255]
    RAW8,
    /// 16bit raw data, 1 pixel 2 bytes, value range[0, 65535]
    RAW16,
    /// RGB888 color data, 1 pixel 3 bytes, value range[0, 255] (only color camera)
    RGB24,
    /// 8bit monochrome data, convert the Bayer Filter Array to monochrome data. 1 pixel 1 byte, value range[0, 255] (only color camera)
    MONO8,
}

impl From<_POAImgFormat> for ImgFormat {
    fn from(value: _POAImgFormat) -> Self {
        use ImgFormat::*;
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

impl Into<_POAImgFormat> for ImgFormat {
    fn into(self) -> _POAImgFormat {
        use ImgFormat::*;
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

pub struct ConfigAttribute<T> {
    pub kind: ConfigKind,
    pub is_support_auto: bool,
    pub is_writable: bool,
    pub is_readable: bool,
    min: T,
    max: T,
    default: T,
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

impl<T: FromAttribute> From<POAConfigAttributes> for ConfigAttribute<T> {
    fn from(value: POAConfigAttributes) -> Self {
        let (min, max, default) = T::from_attribute(value);

        Self {
            kind: value.configID.into(),
            is_support_auto: value.isSupportAuto == POABool::POA_TRUE,
            is_writable: value.isWritable == POABool::POA_TRUE,
            is_readable: value.isReadable == POABool::POA_TRUE,
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

pub struct AllConfigAttributes {
    pub exposure: ConfigAttribute<i64>,
    pub gain: ConfigAttribute<f64>,
    pub hardware_bin: ConfigAttribute<i64>,
    pub temperature: ConfigAttribute<f64>,
    pub wb_r: ConfigAttribute<f64>,
    pub wb_g: ConfigAttribute<f64>,
    pub wb_b: ConfigAttribute<f64>,
    pub offset: ConfigAttribute<i64>,
    pub autoexpo_max_gain: ConfigAttribute<f64>,
    pub autoexpo_max_exposure: ConfigAttribute<i64>,
    pub autoexpo_brightness: ConfigAttribute<i64>,
    pub guide_north: ConfigAttribute<i64>,
    pub guide_south: ConfigAttribute<i64>,
    pub guide_east: ConfigAttribute<i64>,
    pub guide_west: ConfigAttribute<i64>,
    pub egain: ConfigAttribute<f64>,
    pub cooler_power: ConfigAttribute<i64>,
    pub target_temp: ConfigAttribute<i64>,
    pub cooler: ConfigAttribute<bool>,
    pub heater: ConfigAttribute<bool>,
    pub heater_power: ConfigAttribute<i64>,
    pub fan_power: ConfigAttribute<i64>,
    pub flip_none: ConfigAttribute<bool>,
    pub flip_hori: ConfigAttribute<bool>,
    pub flip_vert: ConfigAttribute<bool>,
    pub flip_both: ConfigAttribute<bool>,
    pub frame_limit: ConfigAttribute<i64>,
    pub hqi: ConfigAttribute<bool>,
    pub usb_bandwidth_limit: ConfigAttribute<i64>,
    pub pixel_bin_sum: ConfigAttribute<bool>,
    pub mono_bin: ConfigAttribute<bool>,
}

impl From<Vec<POAConfigAttributes>> for AllConfigAttributes {
    fn from(values: Vec<POAConfigAttributes>) -> Self {
        let mut exposure: Option<ConfigAttribute<i64>> = None;
        let mut gain: Option<ConfigAttribute<f64>> = None;
        let mut hardware_bin: Option<ConfigAttribute<i64>> = None;
        let mut temperature: Option<ConfigAttribute<f64>> = None;
        let mut wb_r: Option<ConfigAttribute<f64>> = None;
        let mut wb_g: Option<ConfigAttribute<f64>> = None;
        let mut wb_b: Option<ConfigAttribute<f64>> = None;
        let mut offset: Option<ConfigAttribute<i64>> = None;
        let mut autoexpo_max_gain: Option<ConfigAttribute<f64>> = None;
        let mut autoexpo_max_exposure: Option<ConfigAttribute<i64>> = None;
        let mut autoexpo_brightness: Option<ConfigAttribute<i64>> = None;
        let mut guide_north: Option<ConfigAttribute<i64>> = None;
        let mut guide_south: Option<ConfigAttribute<i64>> = None;
        let mut guide_east: Option<ConfigAttribute<i64>> = None;
        let mut guide_west: Option<ConfigAttribute<i64>> = None;
        let mut egain: Option<ConfigAttribute<f64>> = None;
        let mut cooler_power: Option<ConfigAttribute<i64>> = None;
        let mut target_temp: Option<ConfigAttribute<i64>> = None;
        let mut cooler: Option<ConfigAttribute<bool>> = None;
        let mut heater: Option<ConfigAttribute<bool>> = None;
        let mut heater_power: Option<ConfigAttribute<i64>> = None;
        let mut fan_power: Option<ConfigAttribute<i64>> = None;
        let mut flip_none: Option<ConfigAttribute<bool>> = None;
        let mut flip_hori: Option<ConfigAttribute<bool>> = None;
        let mut flip_vert: Option<ConfigAttribute<bool>> = None;
        let mut flip_both: Option<ConfigAttribute<bool>> = None;
        let mut frame_limit: Option<ConfigAttribute<i64>> = None;
        let mut hqi: Option<ConfigAttribute<bool>> = None;
        let mut usb_bandwidth_limit: Option<ConfigAttribute<i64>> = None;
        let mut pixel_bin_sum: Option<ConfigAttribute<bool>> = None;
        let mut mono_bin: Option<ConfigAttribute<bool>> = None;

        for value in values {
            let kind = value.configID.into();
            match kind {
                ConfigKind::Exposure => {
                    exposure = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Gain => {
                    gain = Some(ConfigAttribute::from(value));
                }
                ConfigKind::HardwareBin => {
                    hardware_bin = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Temperature => {
                    temperature = Some(ConfigAttribute::from(value));
                }
                ConfigKind::WbR => {
                    wb_r = Some(ConfigAttribute::from(value));
                }
                ConfigKind::WbG => {
                    wb_g = Some(ConfigAttribute::from(value));
                }
                ConfigKind::WbB => {
                    wb_b = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Offset => {
                    offset = Some(ConfigAttribute::from(value));
                }
                ConfigKind::AutoexpoMaxGain => {
                    autoexpo_max_gain = Some(ConfigAttribute::from(value));
                }
                ConfigKind::AutoexpoMaxExposure => {
                    autoexpo_max_exposure = Some(ConfigAttribute::from(value));
                }
                ConfigKind::AutoexpoBrightness => {
                    autoexpo_brightness = Some(ConfigAttribute::from(value));
                }
                ConfigKind::GuideNorth => {
                    guide_north = Some(ConfigAttribute::from(value));
                }
                ConfigKind::GuideSouth => {
                    guide_south = Some(ConfigAttribute::from(value));
                }
                ConfigKind::GuideEast => {
                    guide_east = Some(ConfigAttribute::from(value));
                }
                ConfigKind::GuideWest => {
                    guide_west = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Egain => {
                    egain = Some(ConfigAttribute::from(value));
                }
                ConfigKind::CoolerPower => {
                    cooler_power = Some(ConfigAttribute::from(value));
                }
                ConfigKind::TargetTemp => {
                    target_temp = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Cooler => {
                    cooler = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Heater => {
                    heater = Some(ConfigAttribute::from(value));
                }
                ConfigKind::HeaterPower => {
                    heater_power = Some(ConfigAttribute::from(value));
                }
                ConfigKind::FanPower => {
                    fan_power = Some(ConfigAttribute::from(value));
                }
                ConfigKind::FlipNone => {
                    flip_none = Some(ConfigAttribute::from(value));
                }
                ConfigKind::FlipHori => {
                    flip_hori = Some(ConfigAttribute::from(value));
                }
                ConfigKind::FlipVert => {
                    flip_vert = Some(ConfigAttribute::from(value));
                }
                ConfigKind::FlipBoth => {
                    flip_both = Some(ConfigAttribute::from(value));
                }
                ConfigKind::FrameLimit => {
                    frame_limit = Some(ConfigAttribute::from(value));
                }
                ConfigKind::Hqi => {
                    hqi = Some(ConfigAttribute::from(value));
                }
                ConfigKind::UsbBandwidthLimit => {
                    usb_bandwidth_limit = Some(ConfigAttribute::from(value));
                }
                ConfigKind::PixelBinSum => {
                    pixel_bin_sum = Some(ConfigAttribute::from(value));
                }
                ConfigKind::MonoBin => {
                    mono_bin = Some(ConfigAttribute::from(value));
                }
            }
        }

        Self {
            exposure: exposure.expect("exposure is not found"),
            gain: gain.expect("gain is not found"),
            hardware_bin: hardware_bin.expect("hardware_bin is not found"),
            temperature: temperature.expect("temperature is not found"),
            wb_r: wb_r.expect("wb_r is not found"),
            wb_g: wb_g.expect("wb_g is not found"),
            wb_b: wb_b.expect("wb_b is not found"),
            offset: offset.expect("offset is not found"),
            autoexpo_max_gain: autoexpo_max_gain.expect("autoexpo_max_gain is not found"),
            autoexpo_max_exposure: autoexpo_max_exposure
                .expect("autoexpo_max_exposure is not found"),
            autoexpo_brightness: autoexpo_brightness.expect("autoexpo_brightness is not found"),
            guide_north: guide_north.expect("guide_north is not found"),
            guide_south: guide_south.expect("guide_south is not found"),
            guide_east: guide_east.expect("guide_east is not found"),
            guide_west: guide_west.expect("guide_west is not found"),
            egain: egain.expect("egain is not found"),
            cooler_power: cooler_power.expect("cooler_power is not found"),
            target_temp: target_temp.expect("target_temp is not found"),
            cooler: cooler.expect("cooler is not found"),
            heater: heater.expect("heater is not found"),
            heater_power: heater_power.expect("heater_power is not found"),
            fan_power: fan_power.expect("fan_power is not found"),
            flip_none: flip_none.expect("flip_none is not found"),
            flip_hori: flip_hori.expect("flip_hori is not found"),
            flip_vert: flip_vert.expect("flip_vert is not found"),
            flip_both: flip_both.expect("flip_both is not found"),
            frame_limit: frame_limit.expect("frame_limit is not found"),
            hqi: hqi.expect("hqi is not found"),
            usb_bandwidth_limit: usb_bandwidth_limit.expect("usb_bandwidth_limit is not found"),
            pixel_bin_sum: pixel_bin_sum.expect("pixel_bin_sum is not found"),
            mono_bin: mono_bin.expect("mono_bin is not found"),
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
