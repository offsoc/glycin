use crate::config;
use crate::dbus::*;
use gio::prelude::*;
use glycin_utils::*;

pub use crate::dbus::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Image request builder
#[derive(Debug)]
pub struct ImageRequest {
    file: gio::File,
    mime_type: Option<glib::GString>,
    cancellable: Option<gio::Cancellable>,
}

impl ImageRequest {
    pub fn new(file: gio::File) -> Self {
        Self {
            file,
            mime_type: None,
            cancellable: None,
        }
    }

    pub fn cancellable(mut self, cancellable: impl IsA<gio::Cancellable>) -> Self {
        self.cancellable = Some(cancellable.upcast());
        self
    }

    pub async fn request<'a>(mut self) -> Result<Image<'a>> {
        let config = config::Config::load();

        let gfile_worker = GFileWorker::spawn(self.file.clone(), self.cancellable.clone());
        let mime_type = Self::guess_mime_type(&gfile_worker).await?;

        let process = DecoderProcess::new(&mime_type, &config, self.cancellable.as_ref()).await?;
        let info = process.init(gfile_worker).await?;

        self.mime_type = Some(mime_type);

        Ok(Image {
            process,
            info,
            request: self,
        })
    }

    async fn guess_mime_type(gfile_worker: &GFileWorker) -> Result<glib::GString> {
        let head = gfile_worker.head().await?;
        let (content_type, unsure) = gio::content_type_guess(None::<String>, &head);
        let mime_type = gio::content_type_get_mime_type(&content_type)
            .ok_or_else(|| Error::UnknownImageFormat(content_type.to_string()));

        // Prefer file extension for TIFF since it can be a RAW format as well
        let is_tiff = mime_type.clone().ok() == Some("image/tiff".into());

        if unsure || is_tiff {
            if let Some(filename) = gfile_worker.file().basename() {
                let content_type_fn = gio::content_type_guess(Some(filename), &head).0;
                return gio::content_type_get_mime_type(&content_type_fn)
                    .ok_or_else(|| Error::UnknownImageFormat(content_type_fn.to_string()));
            }
        }

        mime_type
    }
}

/// Image handle containing metadata and allowing frame requests
#[derive(Debug)]
pub struct Image<'a> {
    request: ImageRequest,
    process: DecoderProcess<'a>,
    info: ImageInfo,
}

impl<'a> Image<'a> {
    pub async fn next_frame(&self) -> Result<Frame> {
        self.process.decode_frame().await.map_err(Into::into)
    }

    pub async fn texture(self) -> Result<gdk::Texture> {
        self.process
            .decode_frame()
            .await
            .map(|x| x.texture)
            .map_err(Into::into)
    }

    pub fn info(&self) -> &ImageInfo {
        &self.info
    }

    pub fn request(&self) -> &ImageRequest {
        &self.request
    }
}

impl Drop for ImageRequest {
    fn drop(&mut self) {
        if let Some(cancellable) = &self.cancellable {
            cancellable.cancel();
        }
    }
}

pub struct Frame {
    pub texture: gdk::Texture,
    pub delay: Option<std::time::Duration>,
}
