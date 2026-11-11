fn main() {
    #[cfg(target_os = "windows")]
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set("CompanyName", "The Yellow Mumba Community");
        res.compile().unwrap();
    }

    #[cfg(feature = "git")]
    built::write_built_file().expect("Failed to acquire build-time information");
}
