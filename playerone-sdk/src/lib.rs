use std::collections::HashMap;

use playerone_sdk_sys::{
    _POAErrors, POACameraProperties, POACameraState, POACloseCamera, POAConfigAttributes,
    POAErrors, POAGetCameraCount, POAGetCameraProperties, POAGetCameraState,
    POAGetConfigAttributes, POAGetConfigsCount, POAGetErrorString, POAGetImageSize,
    POAGetImageStartPos, POAInitCamera, POAOpenCamera, POASetImageFormat, POASetImageSize,
    POASetImageStartPos, POAStopExposure,
};

use crate::types::{AllConfigAttributes, ImgFormat};

mod types;

#[derive(Debug, Clone)]
pub struct ROIArea {
    pub start_x: i32,
    pub start_y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct POACamera {
    camera_id: i32,
    closed: bool,
}

impl Drop for POACamera {
    fn drop(&mut self) {
        if !self.closed {
            let _ = self.close();
        }
    }
}

impl POACamera {
    pub fn new(camera_id: i32) -> Self {
        Self {
            camera_id,
            closed: false,
        }
    }

    pub fn get_all_camera_id_name() -> Result<HashMap<i32, String>, String> {
        let mut camera_id_name = HashMap::new();
        let camera_count = unsafe { POAGetCameraCount() };

        for i in 0..camera_count {
            let mut camera_prop: POACameraProperties = POACameraProperties::default();
            let error = unsafe { POAGetCameraProperties(i, &mut camera_prop) };

            if error != _POAErrors::POA_OK {
                continue;
            }
            let name = unsafe { std::ffi::CStr::from_ptr(camera_prop.cameraModelName.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            camera_id_name.insert(camera_prop.cameraID, name);
        }

        Ok(camera_id_name)
    }

    pub fn open(&mut self) -> Result<(), String> {
        let error = unsafe { POAOpenCamera(self.camera_id) };
        if error == _POAErrors::POA_OK {
            Ok(())
        } else {
            Err(self.get_error_string(error))
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        let error = unsafe { POAInitCamera(self.camera_id) };
        if error == _POAErrors::POA_OK {
            Ok(())
        } else {
            Err(self.get_error_string(error))
        }
    }

    pub fn close(&mut self) -> Result<(), String> {
        if self.closed {
            return Ok(());
        }
        let error = unsafe { POACloseCamera(self.camera_id) };
        if error == _POAErrors::POA_OK {
            self.closed = true;
            Ok(())
        } else {
            Err(self.get_error_string(error))
        }
    }

    pub fn get_all_config_attributes(&self) -> Result<AllConfigAttributes, String> {
        let mut config_count = 0;
        let error = unsafe { POAGetConfigsCount(self.camera_id, &mut config_count) };
        if error != _POAErrors::POA_OK {
            return Err(self.get_error_string(error));
        }

        let mut attributes = Vec::with_capacity(40);

        for i in 0..config_count {
            let mut conf_attributes = POAConfigAttributes::default();

            let error = unsafe { POAGetConfigAttributes(self.camera_id, i, &mut conf_attributes) };
            if error != _POAErrors::POA_OK {
                return Err(self.get_error_string(error));
            }

            attributes.push(conf_attributes);
        }

        Ok(AllConfigAttributes::from(attributes))
    }

    pub fn set_roi_area(&self, roi_area: &ROIArea) -> Result<(), String> {
        let mut camera_state = POACameraState::STATE_CLOSED;
        unsafe { POAGetCameraState(self.camera_id, &mut camera_state) };

        if camera_state == POACameraState::STATE_EXPOSING {
            unsafe { POAStopExposure(self.camera_id) };
        }

        let error = unsafe { POASetImageSize(self.camera_id, roi_area.width, roi_area.height) };
        if error != _POAErrors::POA_OK {
            return Err(self.get_error_string(error));
        }

        let error =
            unsafe { POASetImageStartPos(self.camera_id, roi_area.start_x, roi_area.start_y) };
        if error != _POAErrors::POA_OK {
            return Err(self.get_error_string(error));
        }

        Ok(())
    }

    pub fn get_roi_area(&self) -> Result<ROIArea, String> {
        let mut roi_area = ROIArea {
            start_x: 0,
            start_y: 0,
            width: 0,
            height: 0,
        };

        let error = unsafe {
            POAGetImageStartPos(self.camera_id, &mut roi_area.start_x, &mut roi_area.start_y)
        };
        if error != _POAErrors::POA_OK {
            return Err(self.get_error_string(error));
        }

        let error =
            unsafe { POAGetImageSize(self.camera_id, &mut roi_area.width, &mut roi_area.height) };
        if error != _POAErrors::POA_OK {
            return Err(self.get_error_string(error));
        }

        Ok(roi_area)
    }

    pub fn set_image_format(&self, img_fmt: ImgFormat) -> Result<(), String> {
        let poa_img_fmt = img_fmt.into();

        let error = unsafe { POASetImageFormat(self.camera_id, poa_img_fmt) };
        if error == _POAErrors::POA_OK {
            Ok(())
        } else {
            Err(self.get_error_string(error))
        }
    }

    fn get_error_string(&self, error: POAErrors) -> String {
        unsafe { std::ffi::CStr::from_ptr(POAGetErrorString(error)) }
            .to_string_lossy()
            .into_owned()
    }
}
