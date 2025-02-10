pub use bindings::*;

#[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

impl From<bool> for POABool {
    fn from(value: bool) -> Self {
        match value {
            true => POABool::POA_TRUE,
            false => POABool::POA_FALSE,
        }
    }
}

impl From<POABool> for bool {
    fn from(value: POABool) -> Self {
        match value {
            POABool::POA_TRUE => true,
            POABool::POA_FALSE => false,
        }
    }
}

impl From<f64> for POAConfigValue {
    fn from(value: f64) -> Self {
        POAConfigValue { floatValue: value }
    }
}

impl From<i64> for POAConfigValue {
    fn from(value: i64) -> Self {
        POAConfigValue { intValue: value }
    }
}

impl From<bool> for POAConfigValue {
    fn from(value: bool) -> Self {
        POAConfigValue {
            boolValue: value.into(),
        }
    }
}

pub unsafe trait FromPOAConfigValue {
    fn from_poa_config_value(value: POAConfigValue) -> Self;
}

unsafe impl FromPOAConfigValue for f64 {
    fn from_poa_config_value(value: POAConfigValue) -> Self {
        unsafe { value.floatValue }
    }
}

unsafe impl FromPOAConfigValue for i64 {
    fn from_poa_config_value(value: POAConfigValue) -> Self {
        unsafe { value.intValue }
    }
}

unsafe impl FromPOAConfigValue for bool {
    fn from_poa_config_value(value: POAConfigValue) -> Self {
        unsafe { value.boolValue.into() }
    }
}
