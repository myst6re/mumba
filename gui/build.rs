fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();

    #[cfg(target_os = "windows")]
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("data/icon.ico");
        res.set("CompanyName", "The Yellow Mumba Community");
        res.compile().unwrap();
    }

    built::write_built_file().expect("Failed to acquire build-time information");
}
