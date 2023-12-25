use std::os::unix::net::UnixStream;
use std::sync::Mutex;

const INTERFACE_NAME: &str = "org.gnome.glycin.Loader";

#[test]
#[ignore]
fn dbus_api_stability() {
    async_std::task::spawn(start_dbus());
    let output = std::process::Command::new("busctl")
        .args([
            "introspect",
            "--user",
            "--xml-interface",
            "org.gnome.glycin.Test",
            "/org/gnome/glycin/test",
        ])
        .output()
        .unwrap();

    let compat_version = glycin::COMPAT_VERSION;
    let current_api =
        std::fs::read_to_string(format!("../docs/{compat_version}+/{INTERFACE_NAME}.xml")).unwrap();

    let s = r#"<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
  "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
"#
    .to_string();

    let mut api = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .fold((false, s), |(mut take, mut s), line| {
            if line.contains(INTERFACE_NAME) {
                take = true;
            }

            if take {
                s.push_str(line);
                s.push('\n');
            }

            if line.contains("</interface>") {
                take = false;
            }

            (take, s)
        })
        .1;

    api.push_str("</node>\n");

    if current_api != api {
        eprintln!("{api}");
    }

    assert_eq!(api, current_api);
}

async fn start_dbus() {
    struct MockDecoder {}

    impl glycin_utils::Decoder for MockDecoder {
        fn init(
            &self,
            _stream: UnixStream,
            _mime_type: String,
            _details: glycin_utils::InitializationDetails,
        ) -> Result<glycin_utils::ImageInfo, glycin_utils::DecoderError> {
            unimplemented!()
        }
        fn decode_frame(
            &self,
            _frame_request: glycin_utils::FrameRequest,
        ) -> Result<glycin_utils::Frame, glycin_utils::DecoderError> {
            unimplemented!()
        }
    }

    let decoder = MockDecoder {};

    let instruction_handler = glycin_utils::Decoding {
        decoder: Mutex::new(Box::new(decoder)),
    };

    let _connection = zbus::ConnectionBuilder::session()
        .unwrap()
        .name("org.gnome.glycin.Test")
        .unwrap()
        .serve_at("/org/gnome/glycin/test", instruction_handler)
        .unwrap()
        .build()
        .await
        .unwrap();

    std::future::pending::<()>().await;
}
