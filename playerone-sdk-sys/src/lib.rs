#[allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;