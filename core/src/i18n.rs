use fluent_bundle::{FluentBundle, FluentResource};
use std::path::PathBuf;
use unic_langid::{langid, LanguageIdentifier};

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new() -> I18n {
        let mut lang = Self::detect_system_lang();
        let mut path = Self::find_path("mumba", &lang);
        let fallback_land = langid!("en");
        let fallback = Self::find_path("mumba", &fallback_land);
        if !path.exists() {
            lang = fallback_land;
            path = fallback.clone()
        }

        Self::from_file(&path, lang, &fallback)
    }

    pub fn from_file(path: &PathBuf, lang: LanguageIdentifier, fallback: &PathBuf) -> I18n {
        let resource = Self::open_resource(path);
        let resource_fallback = Self::open_resource(fallback);
        let mut bundle = FluentBundle::new(vec![lang, langid!("en")]);

        bundle.add_resource_overriding(resource_fallback);
        bundle.add_resource_overriding(resource);

        I18n { bundle }
    }

    fn open_resource(path: &PathBuf) -> FluentResource {
        let content = std::fs::read_to_string(path)
            .expect(format!("Cannot read fluent file {}", path.to_string_lossy()).as_str());
        FluentResource::try_new(content).expect("Failed to parse an FTL string.")
    }

    pub fn tr(&self, id: &str) -> String {
        let mut errors = vec![];
        let message = self.bundle.get_message(id);
        let message = if cfg!(debug_assertions) {
            message.expect(format!("Fluent: No message found for {}", id).as_str())
        } else {
            return String::from(id);
        };
        let pattern = message.value();
        let pattern = if cfg!(debug_assertions) {
            pattern.expect(format!("Fluent: Message has no value for {}", id).as_str())
        } else {
            return String::from(id);
        };
        String::from(self.bundle.format_pattern(&pattern, None, &mut errors))
    }

    pub fn find_path(name: &str, lang: &LanguageIdentifier) -> PathBuf {
        let file_name = format!("{}.{}.ftl", name, lang);
        PathBuf::from("lang").join(file_name)
    }

    pub fn detect_system_lang() -> LanguageIdentifier {
        langid!("fr")
    }
}
