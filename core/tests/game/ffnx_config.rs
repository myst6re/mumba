use mumba_core::game::ffnx_config::FfnxConfig;

#[test]
pub fn pack_not_exists_file() {
    let config = FfnxConfig::from_file(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/ffnx.toml"),
    );
    if let Ok(config) = config {
        assert_eq!(
            config.get_string("app_path", "foobar").unwrap(),
            "/app/path"
        );
        assert_eq!(config.get_bool("enable_lighting", true).unwrap(), false);
        assert_eq!(config.get_bool("toto", true).unwrap(), true);
        assert_eq!(config.get_int("renderer_backend", -1).unwrap(), 0);
        assert_eq!(config.get_int("game_lighting", -1).unwrap(), 1);
        assert_eq!(config.get_int("foobar", -1).unwrap(), -1);
        assert_eq!(config.get_string("foobar", "test").unwrap(), "test");
        assert_eq!(config.get_string("direct_mode_path", "").unwrap(), "direct");
    } else {
        assert!(false, "Cannot open file");
    }
}
