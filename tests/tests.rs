use gdk::prelude::*;
use std::path::{Path, PathBuf};

#[test]
fn color() {
    async_std::task::block_on(test_dir("test-images/images/color"));
}

#[test]
fn color_iccp_pro() {
    async_std::task::block_on(test_dir("test-images/images/color-iccp-pro"));
}

#[test]
fn gray_iccp() {
    async_std::task::block_on(test_dir("test-images/images/gray-iccp"));
}

async fn test_dir(dir: impl AsRef<Path>) {
    let images = std::fs::read_dir(&dir).unwrap();

    let mut reference_path = dir.as_ref().to_path_buf();
    reference_path.set_extension("png");

    let mut some_failed = false;
    let mut list = Vec::new();
    for entry in images {
        let path = entry.unwrap().path();

        let (failed, deviation) = compare_images(&reference_path, &path).await;

        list.push((format!("{path:#?}"), failed, deviation));

        if failed {
            some_failed = true;
        }
    }

    assert!(!some_failed, "{list:#?}");
}

async fn compare_images(reference_path: impl AsRef<Path>, path: impl AsRef<Path>) -> (bool, f64) {
    let reference_data = get_downloaded_texture(&reference_path).await;
    let data = get_downloaded_texture(&path).await;

    assert_eq!(reference_data.len(), data.len());

    let len = data.len();

    let mut dev = 0;
    for (r, p) in reference_data.into_iter().zip(&data) {
        dev += (r as i16 - *p as i16).abs() as u64;
    }

    let deviation = dev as f64 / len as f64;

    let failed = deviation > 3.1;

    if failed {
        debug_file(&path).await;
    }

    (failed, deviation)
}

async fn get_downloaded_texture(path: impl AsRef<Path>) -> Vec<u8> {
    let texture = get_texture(&path).await;
    let mut data = vec![0; texture.width() as usize * texture.height() as usize * 4];
    texture.download(&mut data, texture.width() as usize * 4);
    data
}

async fn debug_file(path: impl AsRef<Path>) {
    let texture = get_texture(&path).await;
    let mut new_path = PathBuf::from("failures");
    new_path.push(path.as_ref().file_name().unwrap());
    let mut extension = new_path.extension().unwrap().to_os_string();
    extension.push(".png");
    new_path.set_extension(extension);
    texture.save_to_png(new_path).unwrap();
}

async fn get_texture(path: impl AsRef<Path>) -> gdk::Texture {
    let file = gio::File::for_path(&path);
    let image_request = glycin::ImageRequest::new(file);
    let image = image_request.request().await.unwrap();
    let frame = image.next_frame().await.unwrap();
    frame.texture
}
