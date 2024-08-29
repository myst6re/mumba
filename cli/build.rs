extern crate winresource;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set("CompanyName", "The Yellow Mumba Community");
        res.compile().unwrap();
    }

    built::write_built_file().expect("Failed to acquire build-time information");
}
