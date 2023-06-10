use gio::glib;
use std::collections::HashMap;
use std::ffi::OsStr;

pub type MimeType = String;

const CONFIG_FILE_EXT: &str = "conf";
const API_VERSION: u8 = 0;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub image_decoders: HashMap<MimeType, ImageDecoderConfig>,
}

#[derive(Debug, Clone)]
pub struct ImageDecoderConfig {
    pub exec: std::path::PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let mut config = Config::default();

        let mut data_dirs = glib::system_data_dirs();
        data_dirs.push(glib::user_data_dir());

        for mut data_dir in data_dirs {
            data_dir.push("glycin");
            data_dir.push(format!("{API_VERSION}+"));
            data_dir.push("conf.d");

            if let Ok(config_files) = std::fs::read_dir(data_dir) {
                for entry in config_files.flatten() {
                    if entry.path().extension() == Some(OsStr::new(CONFIG_FILE_EXT)) {
                        if let Err(err) = Self::load_file(&entry.path(), &mut config) {
                            eprintln!("Failed to load config file: {err}");
                        }
                    }
                }
            }
        }

        config
    }

    pub fn load_file(path: &std::path::Path, config: &mut Config) -> Result<(), glib::Error> {
        let keyfile = glib::KeyFile::new();
        keyfile.load_from_file(path, glib::KeyFileFlags::NONE)?;

        for group in keyfile.groups() {
            let mut elements = group.to_str().split(':');
            let kind = elements.next();
            let mime_type = elements.next();

            if kind == Some("loader") {
                if let Some(mime_type) = mime_type {
                    if let Ok(exec) = keyfile.string(group.to_str().trim(), "exec") {
                        let cfg = ImageDecoderConfig { exec: exec.into() };

                        config.image_decoders.insert(mime_type.to_string(), cfg);
                    }
                }
            }
        }

        Ok(())
    }
}
