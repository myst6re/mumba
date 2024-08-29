use crate::provision;
use jiff::Timestamp;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use std::fmt;
use std::marker::PhantomData;

#[derive(Deserialize, Clone)]
pub struct GitHubReleaseAsset {
    pub browser_download_url: String,
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub id: i64,
    pub assets: Vec<GitHubReleaseAsset>,
    pub prerelease: bool,
    pub draft: bool,
    pub published_at: String,
}

pub struct LatestRelease {
    pub latest: Option<GitHubRelease>,
    pub prerelease: Option<GitHubRelease>,
    pub latest_not_recent: Option<GitHubRelease>,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct GitHubReleases {
    #[serde(deserialize_with = "deserialize_max")]
    latest_release: LatestRelease,
}

fn deserialize_max<'de, D>(deserializer: D) -> Result<LatestRelease, D::Error>
where
    D: Deserializer<'de>,
{
    struct MaxVisitor(PhantomData<fn() -> LatestRelease>);

    impl<'de> Visitor<'de> for MaxVisitor {
        type Value = LatestRelease;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<LatestRelease, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut latest = None;
            let mut prerelease = None;

            while let Some(value) = seq.next_element::<GitHubRelease>()? {
                if value.prerelease {
                    prerelease = Some(value)
                } else if !value.draft {
                    latest = match &latest {
                        None => Some(value),
                        Some(v) => {
                            let published_at: Timestamp = value
                                .published_at
                                .parse()
                                .map_err(|_| de::Error::custom("Cannot parse date"))?;
                            let current_published_at: Timestamp = v
                                .published_at
                                .parse()
                                .map_err(|_| de::Error::custom("Cannot parse date"))?;
                            if published_at > current_published_at {
                                Some(value)
                            } else {
                                latest
                            }
                        }
                    }
                }
            }

            Ok(LatestRelease {
                latest_not_recent: latest.clone(),
                latest,
                prerelease,
            })
        }
    }

    let visitor = MaxVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}

pub fn find_last_release(repo_name: &str) -> Result<LatestRelease, provision::ToJsonErrorBox> {
    let releases = provision::get_json::<GitHubReleases>(
        format!("https://api.github.com/repos/{}/releases", repo_name).as_str(),
    )?;
    Ok(releases.latest_release)
}
