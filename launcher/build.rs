extern crate winresource;

fn get_repo_tag() -> String {
    let repo = git2::Repository::discover(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"),
    )
    .unwrap();
    let mut desc_opt = git2::DescribeOptions::new();
    desc_opt.describe_tags().show_commit_oid_as_fallback(true);
    repo.describe(&desc_opt)
        .and_then(|desc| desc.format(None))
        .unwrap()
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../gui/data/icon.ico");
        res.set("CompanyName", "The Yellow Mumba Community");
        res.set("LegalCopyright", &get_repo_tag());
        res.compile().unwrap();
    };

    built::write_built_file().expect("Failed to acquire build-time information");
}
