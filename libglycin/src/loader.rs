use std::ptr;

use gdk::{gio, glib};
use gio::ffi::{GAsyncReadyCallback, GAsyncResult, GTask};
use gio::prelude::*;
use glib::ffi::{gpointer, GError, GType};
use glib::subclass::prelude::*;
use glib::translate::*;
use glycin::gobject;

use crate::common::*;
use crate::*;

pub use glycin::SandboxSelector as GlySandboxSelector;

pub type GlyLoader = <gobject::loader::imp::GlyLoader as ObjectSubclass>::Instance;

#[no_mangle]
pub unsafe extern "C" fn gly_loader_new(file: *mut gio::ffi::GFile) -> *const GlyLoader {
    let file = gio::File::from_glib_none(file);
    gobject::GlyLoader::new(file).to_glib_full()
}

#[no_mangle]
pub unsafe extern "C" fn gly_loader_set_sandbox_selector(
    loader: *mut GlyLoader,
    sandbox_selector: GlySandboxSelector,
) {
    let obj = gobject::GlyLoader::from_glib_none(loader);
    obj.set_sandbox_selector(sandbox_selector);
}

#[no_mangle]
pub unsafe extern "C" fn gly_loader_load(
    loader: *mut GlyLoader,
    g_error: *mut *mut GError,
) -> *const GlyImage {
    let obj = (*loader).imp().obj();

    let result = async_io::block_on(obj.load());

    match result {
        Ok(image) => image.to_glib_full(),
        Err(err) => {
            set_error(g_error, &err);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn gly_loader_load_async(
    loader: *mut GlyLoader,
    cancellable: *mut gio::ffi::GCancellable,
    callback: GAsyncReadyCallback,
    user_data: gpointer,
) {
    let obj = gobject::GlyLoader::from_glib_none(loader);
    let cancellable = Option::<gio::Cancellable>::from_glib_borrow(cancellable);
    let callback = GAsyncReadyCallbackSend(callback.unwrap());
    let user_data = GPointerSend(user_data);

    if let Some(cancellable) = &*cancellable {
        obj.set_cancellable(cancellable);
    }

    let closure = glib::clone!(@strong obj => move |task: gio::Task<gobject::GlyImage>, _: Option<&gobject::GlyLoader>| {
        let result: *mut gio::ffi::GAsyncResult =
            task.upcast_ref::<gio::AsyncResult>().to_glib_none().0;
        callback.call(obj, result, user_data);
    });

    let task = gio::Task::new(Some(&obj), (*cancellable).as_ref(), closure);

    glib::MainContext::ref_thread_default().spawn_local(async move {
        let res = obj.load().await.map_err(|x| glib_error(&x));
        task.return_result(res);
    });
}

#[no_mangle]
pub unsafe extern "C" fn gly_loader_load_finish(
    _loader: *mut GlyLoader,
    res: *mut GAsyncResult,
    error: *mut *mut GError,
) -> *const GlyImage {
    let task = gio::Task::<gobject::GlyImage>::from_glib_none(res as *mut GTask);

    match task.propagate() {
        Ok(image) => image.to_glib_full(),
        Err(e) => {
            if !error.is_null() {
                *error = e.into_glib_ptr();
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn gly_loader_get_type() -> GType {
    <gobject::GlyLoader as StaticType>::static_type().into_glib()
}

#[no_mangle]
pub extern "C" fn gly_sandbox_selector_get_type() -> GType {
    <GlySandboxSelector as StaticType>::static_type().into_glib()
}
