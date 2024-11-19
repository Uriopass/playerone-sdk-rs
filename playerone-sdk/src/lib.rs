use std::ffi::c_long;

use playerone_sdk_sys::{
    _POAErrors, FromPOAConfigValue, POABool, POACameraProperties, POACameraState, POACloseCamera,
    POAConfig, POAConfigAttributes, POAConfigValue, POAGetCameraCount,
    POAGetCameraProperties, POAGetCameraState, POAGetConfig, POAGetConfigAttributes, POAGetConfigsCount,
    POAGetImageData, POAGetImageFormat, POAGetImageSize, POAGetImageStartPos, POAImageReady,
    POAImgFormat, POAInitCamera, POAOpenCamera, POASetConfig, POASetImageFormat,
    POASetImageSize, POASetImageStartPos, POAStartExposure, POAStopExposure,
};
use playerone_sdk_sys::POABool::POA_FALSE;
use playerone_sdk_sys::POAConfig::{POA_EXPOSURE, POA_GAIN};

use crate::types::{AllConfigBounds, CameraProperties, Error, ImageFormat};

mod types;

/// Region Of Interest
#[derive(Debug, Copy, Clone)]
pub struct ROI {
    pub start_x: i32,
    pub start_y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct Camera {
    camera_id: i32,
    closed: bool,
    properties: CameraProperties,
}

impl Drop for Camera {
    fn drop(&mut self) {
        if !self.closed {
            let _ = self.close();
        }
    }
}

type POAResult<T> = Result<T, Error>;

impl Camera {
    /// Returns the list of all available cameras
    pub fn all_cameras() -> Vec<Camera> {
        let camera_count = unsafe { POAGetCameraCount() };
        let mut cameras = Vec::with_capacity(camera_count as usize);

        for i in 0..camera_count {
            let mut camera_prop: POACameraProperties = POACameraProperties::default();
            let error = unsafe { POAGetCameraProperties(i, &mut camera_prop) };

            if error != _POAErrors::POA_OK {
                continue;
            }
            cameras.push(Camera {
                camera_id: camera_prop.cameraID,
                closed: false,
                properties: camera_prop.into(),
            });
        }

        cameras
    }

    pub fn properties(&self) -> &CameraProperties {
        &self.properties
    }

    pub fn open(&mut self) -> POAResult<()> {
        let error = unsafe { POAOpenCamera(self.camera_id) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        let error = unsafe { POAInitCamera(self.camera_id) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        Ok(())
    }

    pub fn start_exposure(&mut self) -> POAResult<()> {
        let error = unsafe { POAStartExposure(self.camera_id, POA_FALSE) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    /// the image data is available? if true, you can call get_image_data to get image data
    pub fn is_image_ready(&self) -> POAResult<bool> {
        let mut is_img_data_available = POA_FALSE;
        let error = unsafe { POAImageReady(self.camera_id, &mut is_img_data_available) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(is_img_data_available.into())
    }

    /// get image data after exposure, this function will block or wait for the timeout
    ///
    /// the buffer size must be bigger than this: POA_RAW8: width * height, POA_RAW16: width * height * 2, POA_RGB24: width * height * 3
    ///
    /// Note: it's recommended to use is_image_ready function for waiting, if image data 'Is Ready', calling this function will return immediately
    pub fn get_image_data(&self, buffer: &mut [u8], timeout: usize) -> POAResult<()> {
        let error = unsafe {
            POAGetImageData(
                self.camera_id,
                buffer.as_mut_ptr(),
                buffer.len() as c_long,
                timeout as i32,
            )
        };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    pub fn stop_exposure(&mut self) -> POAResult<()> {
        let error = unsafe { POAStopExposure(self.camera_id) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    /// Close the camera. This is done automatically on Camera drop but can be called manually if you wish to handle any errors
    /// that may occur.
    pub fn close(&mut self) -> POAResult<()> {
        if self.closed {
            return Ok(());
        }
        let error = unsafe { POACloseCamera(self.camera_id) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        self.closed = true;
        Ok(())
    }

    pub fn config_bounds(&self) -> POAResult<AllConfigBounds> {
        let mut config_count = 0;
        let error = unsafe { POAGetConfigsCount(self.camera_id, &mut config_count) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        let mut attributes = Vec::with_capacity(40);

        for i in 0..config_count {
            let mut conf_attributes = POAConfigAttributes::default();

            let error = unsafe { POAGetConfigAttributes(self.camera_id, i, &mut conf_attributes) };
            if error != _POAErrors::POA_OK {
                return Err(error.into());
            }

            attributes.push(conf_attributes);
        }

        Ok(AllConfigBounds::from(attributes))
    }

    /// Camera must not be exposing when this is called
    ///
    /// Sets the Region Of Interest
    pub fn set_roi(&mut self, roi_area: &ROI) -> POAResult<()> {
        let mut camera_state = POACameraState::STATE_CLOSED;
        unsafe { POAGetCameraState(self.camera_id, &mut camera_state) };

        if camera_state == POACameraState::STATE_EXPOSING {
            unsafe { POAStopExposure(self.camera_id) };
        }

        let error = unsafe { POASetImageSize(self.camera_id, roi_area.width, roi_area.height) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        let error =
            unsafe { POASetImageStartPos(self.camera_id, roi_area.start_x, roi_area.start_y) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        Ok(())
    }

    /// Gets the Region Of Interest
    pub fn roi(&self) -> POAResult<ROI> {
        let mut roi_area = ROI {
            start_x: 0,
            start_y: 0,
            width: 0,
            height: 0,
        };

        let error = unsafe {
            POAGetImageStartPos(self.camera_id, &mut roi_area.start_x, &mut roi_area.start_y)
        };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        let error =
            unsafe { POAGetImageSize(self.camera_id, &mut roi_area.width, &mut roi_area.height) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        Ok(roi_area)
    }

    /// Must be within the bounds specified in the config
    pub fn set_image_size(&mut self, width: i32, height: i32) -> POAResult<()> {
        let error = unsafe { POASetImageSize(self.camera_id, width, height) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    pub fn image_size(&self) -> POAResult<(i32, i32)> {
        let mut width = 0;
        let mut height = 0;

        let error = unsafe { POAGetImageSize(self.camera_id, &mut width, &mut height) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok((width, height))
    }

    /// Sets the offset/anchor/start position in the image
    pub fn set_image_start_pos(&mut self, start_x: i32, start_y: i32) -> POAResult<()> {
        let error = unsafe { POASetImageStartPos(self.camera_id, start_x, start_y) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    pub fn image_start_pos(&self) -> POAResult<(i32, i32)> {
        let mut start_x = 0;
        let mut start_y = 0;

        let error = unsafe { POAGetImageStartPos(self.camera_id, &mut start_x, &mut start_y) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok((start_x, start_y))
    }

    pub fn set_image_format(&mut self, image_format: ImageFormat) -> POAResult<()> {
        let poa_img_format = image_format.into();

        let error = unsafe { POASetImageFormat(self.camera_id, poa_img_format) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    pub fn image_format(&self) -> POAResult<ImageFormat> {
        let mut poa_img_format = POAImgFormat::POA_END;

        let error = unsafe { POAGetImageFormat(self.camera_id, &mut poa_img_format) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(poa_img_format.into())
    }

    /// Sets the exposure time in microseconds
    pub fn set_exposure(&mut self, exposure_micros: i64, is_auto: bool) -> POAResult<()> {
        self.set_config(POA_EXPOSURE, exposure_micros, is_auto)
    }

    pub fn set_gain(&mut self, gain: i64, is_auto: bool) -> POAResult<()> {
        self.set_config(POA_GAIN, gain, is_auto)
    }

    /// Exposure in microseconds and whether it is auto
    pub fn exposure(&self) -> POAResult<(i64, bool)> {
        unsafe { self.get_config_auto(POA_EXPOSURE) }
    }

    /// Gain and whether it is auto
    pub fn gain(&self) -> POAResult<(i64, bool)> {
        unsafe { self.get_config_auto(POA_GAIN) }
    }

    fn hardware_bin(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_HARDWARE_BIN) }
    }

    /// Current temperature in Celsius
    fn temperature(&self) -> POAResult<f64> {
        unsafe { self.get_config(POAConfig::POA_TEMPERATURE) }
    }

    /// red pixels coefficient of white balance
    fn wb_r(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_WB_R) }
    }

    /// green pixels coefficient of white balance
    fn wb_g(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_WB_G) }
    }

    /// blue pixels coefficient of white balance
    fn wb_b(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_WB_B) }
    }

    fn offset(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_OFFSET) }
    }

    /// maximum gain when auto-adjust
    fn auto_max_gain(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_AUTOEXPO_MAX_GAIN) }
    }

    /// maximum exposure when auto-adjust (in ms)
    fn auto_max_exposure_ms(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_AUTOEXPO_MAX_EXPOSURE) }
    }

    /// target brightness when auto-adjust
    fn auto_target_brightness(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_AUTOEXPO_BRIGHTNESS) }
    }

    /// ST4 guide north, generally, it's DEC+ on the mount
    fn guide_north(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_GUIDE_NORTH) }
    }

    /// ST4 guide south, generally, it's DEC- on the mount
    fn guide_south(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_GUIDE_SOUTH) }
    }

    /// ST4 guide east, generally, it's RA+ on the mount
    fn guide_east(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_GUIDE_EAST) }
    }

    /// ST4 guide west, generally, it's RA- on the mount
    fn guide_west(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_GUIDE_WEST) }
    }

    /// e/ADU, This value will change with gain
    fn egain(&self) -> POAResult<f64> {
        unsafe { self.get_config(POAConfig::POA_EGAIN) }
    }

    /// cooler power percentage[0-100%](only cool camera)
    fn cooler_power(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_COOLER_POWER) }
    }

    /// camera target temperature (in Celsius)
    fn target_temp(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_TARGET_TEMP) }
    }

    /// is cooler(and fan) on or off
    fn cooler(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_COOLER) }
    }

    #[deprecated("unsure why this is deprecated")]
    /// get state of lens heater(on or off)
    fn heater(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_HEATER) }
    }

    /// lens heater power percentage[0-100%]
    fn heater_power(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_HEATER_POWER) }
    }

    /// radiator fan power percentage[0-100%]
    fn fan_power(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_FAN_POWER) }
    }

    /// Range is [0, 2000]
    /// 0 means no limit
    fn frame_limit(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_FRAME_LIMIT) }
    }

    /// High Quality Image, for those without DDR camera(guide camera)
    /// if true, this will reduce the waviness and stripe of the image
    fn hqi(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_HQI) }
    }

    /// 0-100% usage of USB bandwidth
    fn usb_bandwidth_limit(&self) -> POAResult<i64> {
        unsafe { self.get_config(POAConfig::POA_USB_BANDWIDTH_LIMIT) }
    }

    /// take the sum or average of pixels after binning, true is sum and false is average, default is false
    fn pixel_bin_sum(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_PIXEL_BIN_SUM) }
    }

    /// only for color camera, when set to true, pixel binning will use neighbour pixels and image
    /// after binning will lose the bayer pattern
    fn mono_bin(&self) -> POAResult<bool> {
        unsafe { self.get_config(POAConfig::POA_MONO_BIN) }
    }

    pub fn set_hardware_bin(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_HARDWARE_BIN, value, false)
    }

    /// set the red pixels coefficient of white balance
    pub fn set_wb_r(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_WB_R, value, false)
    }

    /// set the green pixels coefficient of white balance
    pub fn set_wb_g(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_WB_G, value, false)
    }

    /// set the blue pixels coefficient of white balance
    pub fn set_wb_b(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_WB_B, value, false)
    }

    pub fn set_offset(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_OFFSET, value, false)
    }

    /// set the max gain when auto-adjust
    pub fn set_auto_max_gain(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_AUTOEXPO_MAX_GAIN, value, false)
    }

    /// set the max exposure when auto-adjust (in ms)
    pub fn set_auto_max_exposure_ms(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_AUTOEXPO_MAX_EXPOSURE, value, false)
    }

    /// set the target brightness when auto-adjust
    pub fn set_auto_target_brightness(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_AUTOEXPO_BRIGHTNESS, value, false)
    }

    /// set ST4 guide north, generally, it's DEC+ on the mount
    pub fn set_guide_north(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_GUIDE_NORTH, value, false)
    }

    /// set ST4 guide south, generally, it's DEC- on the mount
    pub fn set_guide_south(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_GUIDE_SOUTH, value, false)
    }

    /// set ST4 guide east, generally, it's RA+ on the mount
    pub fn set_guide_east(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_GUIDE_EAST, value, false)
    }

    /// set ST4 guide west, generally, it's RA- on the mount
    pub fn set_guide_west(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_GUIDE_WEST, value, false)
    }

    /// set the camera target temperature (in Celsius)
    pub fn set_target_temperature(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_TARGET_TEMP, value, false)
    }

    /// set the cooler(and fan) on or off
    pub fn set_cooler(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_COOLER, value, false)
    }

    /// set the state of lens heater(on or off)
    #[deprecated("unsure why this is deprecated")]
    pub fn set_heater(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_HEATER, value, false)
    }

    /// set the lens heater power percentage[0-100%]
    pub fn set_heater_power(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_HEATER_POWER, value, false)
    }

    /// set the radiator fan power percentage[0-100%]
    pub fn set_fan_power(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_FAN_POWER, value, false)
    }

    /// set the frame limit
    /// Range is [0, 2000]. 0 means no limit
    pub fn set_frame_limit(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_FRAME_LIMIT, value, false)
    }

    /// set High Quality Image, for those without DDR camera(guide camera)
    /// if true, this will reduce the waviness and stripe of the image but frame rate may go down
    /// note: this config has no effect on cameras with DDR
    pub fn set_hqi(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_HQI, value, false)
    }

    /// set the maximum usage of USB bandwidth [0-100%]
    pub fn set_usb_bandwidth_limit(&mut self, value: i64) -> POAResult<()> {
        self.set_config(POAConfig::POA_USB_BANDWIDTH_LIMIT, value, false)
    }

    /// set whether to take the sum or average of pixels after binning, true is sum and false is average, default is false
    pub fn set_pixel_bin_sum(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_PIXEL_BIN_SUM, value, false)
    }

    /// only for color camera: if true,  pixel binning will use neighbour pixels
    /// and image after binning will lose the bayer pattern
    pub fn set_mono_bin(&mut self, value: bool) -> POAResult<()> {
        self.set_config(POAConfig::POA_MONO_BIN, value, false)
    }

    fn set_config(
        &mut self,
        poa_config: POAConfig,

        value: impl Into<POAConfigValue>,
        is_auto: bool,
    ) -> POAResult<()> {
        let value = value.into();
        let error = unsafe { POASetConfig(self.camera_id, poa_config, value, is_auto.into()) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }
        Ok(())
    }

    /// # Unsafe
    ///
    /// The given type must match the actual type of the config value
    unsafe fn get_config_auto<T: FromPOAConfigValue>(
        &self,
        poa_config: POAConfig,
    ) -> POAResult<(T, bool)> {
        let mut config_value = POAConfigValue::default();
        let mut is_auto = POABool::POA_FALSE;

        let error =
            unsafe { POAGetConfig(self.camera_id, poa_config, &mut config_value, &mut is_auto) };
        if error != _POAErrors::POA_OK {
            return Err(error.into());
        }

        Ok((config_value.into(), is_auto.into()))
    }

    /// # Unsafe
    ///
    /// The given type must match the actual type of the config value
    unsafe fn get_config<T: FromPOAConfigValue>(&self, poa_config: POAConfig) -> POAResult<T> {
        self.get_config_auto(poa_config).map(|(value, _)| value)
    }
}
