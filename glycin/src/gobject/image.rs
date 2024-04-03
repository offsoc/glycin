use std::sync::OnceLock;

use gio::{glib, Cancellable};
use glib::subclass::prelude::*;
use glycin_utils::ImageInfo;

use super::GlyFrame;
use crate::Image;

pub mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct GlyImage {
        pub(super) image: OnceLock<Image<'static>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GlyImage {
        const NAME: &'static str = "GlyImage";
        type Type = super::GlyImage;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for GlyImage {}
}

glib::wrapper! {
    /// GObject wrapper for [`Image`]
    pub struct GlyImage(ObjectSubclass<imp::GlyImage>);
}

impl GlyImage {
    pub(crate) fn new(image: Image<'static>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().image.set(image).unwrap();
        obj
    }

    pub fn image_info(&self) -> &ImageInfo {
        self.image().info()
    }

    pub async fn next_frame(&self) -> crate::Result<GlyFrame> {
        Ok(GlyFrame::new(self.image().next_frame().await?))
    }

    pub fn cancellable(&self) -> Cancellable {
        self.image().cancellable()
    }

    pub fn image(&self) -> &Image {
        self.imp().image.get().unwrap()
    }
}
