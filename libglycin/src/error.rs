use std::os::raw::c_int;

use gdk::{gio, glib};
use gio::prelude::*;
use glib::error::ErrorDomain;
use glib::ffi::GError;
use glib::translate::*;

#[derive(Debug, Copy, Clone, glib::Enum, glib::ErrorDomain)]
#[error_domain(name = "gly-loader-error")]
#[repr(C)]
#[enum_type(name = "GlyLoaderError")]
pub enum GlyLoaderError {
    Failed = 0,
    UnknownImageFormat = 1,
}

impl From<&glycin::Error> for GlyLoaderError {
    fn from(value: &glycin::Error) -> Self {
        if value.unsupported_format().is_some() {
            Self::UnknownImageFormat
        } else {
            Self::Failed
        }
    }
}

#[no_mangle]
pub extern "C" fn gly_loader_error_quark() -> glib::ffi::GQuark {
    GlyLoaderError::domain().into_glib()
}

#[no_mangle]
pub unsafe extern "C" fn gly_loader_error_get_type() -> glib::ffi::GType {
    GlyLoaderError::static_type().into_glib()
}

pub unsafe fn set_error(g_error: *mut *mut GError, err: &glycin::Error) {
    let gly_error: GlyLoaderError = err.into();

    glib::ffi::g_set_error_literal(
        g_error,
        GlyLoaderError::domain().into_glib(),
        gly_error.code() as c_int,
        err.to_string().to_glib_none().0,
    );
}

pub unsafe fn glib_error(err: &glycin::Error) -> glib::Error {
    let gly_error: GlyLoaderError = err.into();
    glib::Error::new(gly_error, &err.to_string())
}
