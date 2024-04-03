use std::sync::OnceLock;

use gio::glib;
use glib::subclass::prelude::*;

use crate::Frame;

pub mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct GlyFrame {
        pub(super) frame: OnceLock<Frame>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GlyFrame {
        const NAME: &'static str = "GlyFrame";
        type Type = super::GlyFrame;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for GlyFrame {}
}

glib::wrapper! {
    /// GObject wrapper for [`Frame`]
    pub struct GlyFrame(ObjectSubclass<imp::GlyFrame>);
}

impl GlyFrame {
    pub(crate) fn new(frame: Frame) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().frame.set(frame).unwrap();
        obj
    }

    pub fn texture(&self) -> gdk::Texture {
        self.frame().texture.clone()
    }

    pub fn frame(&self) -> &Frame {
        self.imp().frame.get().unwrap()
    }
}
