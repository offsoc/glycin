use std::sync::Mutex;

use gio::glib;
use glib::prelude::*;
use glib::subclass::prelude::*;

use super::GlyImage;
use crate::{Loader, SandboxSelector};

pub mod imp {
    use super::*;

    #[derive(Default, Debug, glib::Properties)]
    #[properties(wrapper_type = super::GlyLoader)]
    pub struct GlyLoader {
        #[property(get, construct_only)]
        file: Mutex<Option<gio::File>>,
        #[property(get, set)]
        cancellable: Mutex<Option<gio::Cancellable>>,
        #[property(get, set, builder(SandboxSelector::default()))]
        sandbox_selector: Mutex<SandboxSelector>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GlyLoader {
        const NAME: &'static str = "GlyLoader";
        type Type = super::GlyLoader;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for GlyLoader {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    /// GObject wrapper for [`Loader`]
    pub struct GlyLoader(ObjectSubclass<imp::GlyLoader>);
}

impl GlyLoader {
    pub fn new(file: gio::File) -> Self {
        glib::Object::builder().property("file", file).build()
    }

    pub async fn load(&self) -> Result<GlyImage, crate::Error> {
        let mut loader = Loader::new(self.file().unwrap());

        loader.sandbox_mechanism = self.sandbox_selector();

        if let Some(cancellable) = self.cancellable() {
            loader.cancellable(cancellable);
        }

        let image = loader.load().await?;

        Ok(GlyImage::new(image))
    }
}
