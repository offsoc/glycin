use gdk::prelude::*;
use gio::glib;

fn main() {
    glib::MainContext::default().block_on(run()).unwrap();
}

async fn run() -> Result<(), glycin::Error> {
    let mut args = std::env::args();
    let bin = args.next().unwrap();
    let Some(path) = args.next() else {
        eprintln!("Usage: {bin} <IMAGE PATH> [NUMBER FRAMES]");
        std::process::exit(2);
    };
    let n_frames = args.next().and_then(|x| x.parse().ok()).unwrap_or(1);

    let file = gio::File::for_path(path);
    let image = glycin::ImageRequest::new(file).request().await?;

    let info = image.info();

    println!("[info]");
    println!("dimensions = {} x {}", info.width, info.height);
    println!(
        "format_name = {}",
        info.details
            .format_name
            .as_ref()
            .cloned()
            .unwrap_or("-".into())
    );
    println!("exif = {}", info.details.exif.is_some());
    println!("xmp = {}", info.details.xmp.is_some());
    println!(
        "dimensions_text = {}",
        info.details
            .dimensions_text
            .as_ref()
            .cloned()
            .unwrap_or(String::from("-"))
    );
    println!(
        "dimensions_inch = {}",
        info.details
            .dimensions_inch
            .as_ref()
            .map(|(x, y)| format!("{:.3}” x {:.3}”", x, y))
            .unwrap_or("-".into())
    );

    for _ in 0..n_frames {
        let frame = image.next_frame().await.unwrap();
        println!("[[frame]]");
        println!(
            "dimensions = {} x {}",
            frame.texture.width(),
            frame.texture.height()
        );
        println!("format = {:?}", frame.texture.format());
        println!(
            "delay = {}",
            frame
                .delay
                .map(|x| format!("{:#?}", x))
                .unwrap_or("-".into())
        );
        println!(
            "bit_depth = {}",
            frame
                .details
                .bit_depth
                .map(|x| format!("{} bit", x))
                .unwrap_or("-".into())
        );
        println!(
            "alpha_channel = {}",
            frame
                .details
                .alpha_channel
                .map(|x| x.to_string())
                .unwrap_or("-".into())
        );
        println!(
            "grayscale = {}",
            frame
                .details
                .grayscale
                .map(|x| x.to_string())
                .unwrap_or("-".into())
        );
    }

    Ok(())
}
