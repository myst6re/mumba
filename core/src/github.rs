use crate::provision;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use version_compare::{compare, Cmp};

use std::fmt;
use std::marker::PhantomData;

use regex_lite::Regex;

#[derive(Deserialize)]
struct GitHubTag {
    name: String,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct GitHubTags {
    #[serde(deserialize_with = "deserialize_max")]
    max_version: String,
}

fn deserialize_max<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct MaxVisitor(PhantomData<fn() -> String>);

    impl<'de> Visitor<'de> for MaxVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<String, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let version_regex = Regex::new(r"^[vV]?([0-9]+\.)*[0-9]+$").unwrap();
            let mut max = None;

            while let Some(value) = seq.next_element::<GitHubTag>()? {
                if version_regex.is_match(&value.name) {
                    max = match &max {
                        None => Some(value.name),
                        Some(v) => {
                            if compare(v, &value.name) == Ok(Cmp::Lt) {
                                Some(value.name)
                            } else {
                                max
                            }
                        }
                    }
                }
            }

            Ok(String::from(max.ok_or_else(|| {
                de::Error::custom("no values in seq when looking for maximum")
            })?))
        }
    }

    let visitor = MaxVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}

pub fn find_last_tag_version(repo_name: &str) -> Result<String, provision::ToJsonError> {
    let tags = provision::get_json::<GitHubTags>(
        format!("https://api.github.com/repos/{}/tags", repo_name).as_str(),
    )?;
    Ok(tags.max_version)
}
