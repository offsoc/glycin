use gdk::prelude::*;
use glycin::*;

fn main() {
    let images = std::fs::read_dir("images/static").unwrap();

    for entry in images {
        eprintln!("{entry:?}");
        let path = entry.unwrap().path();
        let file = gio::File::for_path(&path);

        async_std::task::block_on(async move {
            let cancellable = gio::Cancellable::new();
            let image_request = ImageRequest::new(file.clone()).cancellable(cancellable);

            let image = image_request.request().await.expect("x");
            let frame = image.next_frame().await.unwrap();

            dbg!("read");
            let texture = gdk::Texture::from_file(&file.clone()).unwrap();

            dbg!("write tiff");
            let mut extension = path.extension().unwrap().to_os_string();
            extension.push(".gtk.tiff");
            let out_path =
                std::path::PathBuf::from_iter(&["out".into(), path.with_extension(extension)]);
            texture.save_to_tiff(out_path).unwrap();

            dbg!("write png");
            let mut extension = path.extension().unwrap().to_os_string();
            extension.push(".gtk.png");
            let out_path =
                std::path::PathBuf::from_iter(&["out".into(), path.with_extension(extension)]);
            texture.save_to_png(out_path).unwrap();

            dbg!("write decoded png");
            let mut extension = path.extension().unwrap().to_os_string();
            extension.push(".png");
            let out_path =
                std::path::PathBuf::from_iter(&["out".into(), path.with_extension(extension)]);
            frame.texture.save_to_png(out_path).unwrap();
        });
    }

    let images = std::fs::read_dir("images/animated").unwrap();

    for entry in images {
        eprintln!("{entry:?}");
        let path = entry.unwrap().path();
        let file = gio::File::for_path(&path);

        async_std::task::block_on(async move {
            let cancellable = gio::Cancellable::new();
            let image_request = ImageRequest::new(file).cancellable(cancellable);

            let image = image_request.request().await.unwrap();

            for i in 1..10 {
                let frame = image.next_frame().await.unwrap();

                let mut extension = path.extension().unwrap().to_os_string();
                extension.push(format!(".{i}.png"));
                let out_path =
                    std::path::PathBuf::from_iter(&["out".into(), path.with_extension(extension)]);

                frame.texture.save_to_png(out_path).unwrap();
            }
        });
    }
}
