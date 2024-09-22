use fluent_bundle::{FluentBundle, FluentResource};
use std::path::PathBuf;
use sys_locale::get_locale;
use unic_langid::{langid, LanguageIdentifier};

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new(lang: Option<String>) -> I18n {
        let lang = lang
            .and_then(|l| LanguageIdentifier::from_bytes(l.as_bytes()).ok())
            .unwrap_or_else(Self::detect_system_lang);
        let path = Self::find_path(&lang);

        Self::from_file(&path, lang)
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P, lang: LanguageIdentifier) -> I18n {
        let fallback_lang = langid!("en-US");
        let fallback = Self::find_path(&fallback_lang);

        info!("Fluent file path: {}", path.as_ref().to_string_lossy());

        let resource = Self::open_resource(path);
        let resource_fallback = Self::open_resource(&fallback);
        let mut bundle = FluentBundle::new(vec![lang.clone(), langid!("en-US")]);

        if let Some(resource_fallback) = resource_fallback {
            bundle.add_resource_overriding(resource_fallback)
        } else {
            error!("Cannot load lang: {}", &fallback_lang);
        }
        if let Some(resource) = resource {
            bundle.add_resource_overriding(resource)
        } else {
            error!("Cannot load lang: {}", &lang);
        }

        I18n { bundle }
    }

    pub fn lang(&self) -> Option<&LanguageIdentifier> {
        self.bundle.locales.first()
    }

    fn open_resource<P: AsRef<std::path::Path>>(path: P) -> Option<FluentResource> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| FluentResource::try_new(content).ok())
    }

    pub fn tr(&self, id: &str) -> String {
        let mut errors = vec![];
        let message = self.bundle.get_message(id);
        let message = if let Some(message) = message {
            message
        } else {
            return String::from(id);
        };
        let pattern = message.value();
        let pattern = if let Some(pattern) = pattern {
            pattern
        } else {
            return String::from(id);
        };
        String::from(self.bundle.format_pattern(pattern, None, &mut errors))
    }

    pub fn find_path(lang: &LanguageIdentifier) -> PathBuf {
        let file_name = format!("mumba.{}.ftl", lang.language);
        let path = PathBuf::from("lang").join(&file_name);
        if path.exists() {
            path
        } else {
            PathBuf::from("/var/lib/mumba/lang").join(file_name)
        }
    }

    pub fn detect_system_lang() -> LanguageIdentifier {
        get_locale()
            .and_then(|locale| LanguageIdentifier::from_bytes(locale.as_bytes()).ok())
            .unwrap_or_else(|| langid!("en-US"))
    }
}
