extern crate winresource;

fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("data/icon.ico");
        res.compile().unwrap();
    }
}
