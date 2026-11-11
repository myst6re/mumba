use fluent_bundle::{FluentBundle, FluentResource};
use std::path::PathBuf;
use sys_locale::get_locale;
use unic_langid::{langid, LanguageIdentifier};

use crate::game::env::Env;

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new(lang: Option<String>, env: &Env) -> I18n {
        let lang = lang
            .and_then(|l| LanguageIdentifier::from_bytes(l.as_bytes()).ok())
            .unwrap_or_else(Self::detect_system_lang);
        let path = Self::build_path(&lang, env);

        Self::from_file(&path, lang, env)
    }

    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
        lang: LanguageIdentifier,
        env: &Env,
    ) -> I18n {
        let fallback_lang = langid!("en-US");
        let fallback = Self::build_path(&fallback_lang, env);

        info!(
            "Fluent file path: \"{}\" (fallback: \"{}\")",
            path.as_ref().to_string_lossy(),
            fallback.to_string_lossy()
        );

        let resource = Self::open_resource(path);
        let resource_fallback = Self::open_resource(&fallback);
        let mut bundle = FluentBundle::new(vec![lang.clone(), langid!("en-US")]);

        if let Some(resource_fallback) = resource_fallback {
            bundle.add_resource_overriding(resource_fallback)
        } else {
            error!("Cannot load lang: {}", fallback_lang);
        }
        if let Some(resource) = resource {
            bundle.add_resource_overriding(resource)
        } else {
            error!("Cannot load lang: {}", lang);
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

    pub fn build_path(lang: &LanguageIdentifier, env: &Env) -> PathBuf {
        env.find_lang(lang.language.as_str())
    }

    pub fn detect_system_lang() -> LanguageIdentifier {
        get_locale()
            .and_then(|locale| LanguageIdentifier::from_bytes(locale.as_bytes()).ok())
            .unwrap_or_else(|| langid!("en-US"))
    }
}
