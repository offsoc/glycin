use gdk::{gio, glib};
use gio::prelude::*;
use glib::ffi::gpointer;

pub struct GPointerSend(pub gpointer);

unsafe impl Send for GPointerSend {}

pub struct GAsyncReadyCallbackSend(
    pub unsafe extern "C" fn(*mut glib::gobject_ffi::GObject, *mut gio::ffi::GAsyncResult, gpointer),
);

unsafe impl Send for GAsyncReadyCallbackSend {}

impl GAsyncReadyCallbackSend {
    pub unsafe fn call<P, O>(
        &self,
        obj: O,
        res: *mut gio::ffi::GAsyncResult,
        user_data: GPointerSend,
    ) where
        O: glib::translate::ToGlibPtr<'static, *mut P> + IsA<glib::Object>,
    {
        let obj_ptr: *mut P = obj.to_glib_full();
        self.0(obj_ptr as *mut _, res, user_data.0)
    }
}
