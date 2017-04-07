
// Standard library
use std::collections::{BTreeMap, HashMap};

// Third party
use chrono_tz::Etc::UTC;
use chrono_tz::Tz;
use yaml_rust::{yaml, Yaml, YamlLoader};

// First party
pub use validated_types::Url as ValidatedUrl;
use yaml_util::*;


#[derive(Debug, PartialEq)]
pub struct SiteInfo {
    /// The name of the site. Required.
    pub title: String,

    /// The canonical URL for the root of the site. Required.
    pub url: ValidatedUrl,

    /// The default timezone to use with posts. May be any of the full text
    /// strings specified by [the `chrono_tz` crate][chrono_tz].
    ///
    /// [chrono_tz]: https://docs.rs/chrono-tz/
    pub default_timezone: Tz,

    /// The description of the site. Optional.
    pub description: Option<String>,

    /// Arbitrary metadata associated with the site. Optional.
    pub metadata: HashMap<String, Yaml>,
}


impl SiteInfo {
    pub fn from_yaml(yaml: &yaml::Hash) -> Result<SiteInfo, String> {
        let title = SiteInfo::parse_title(yaml)?;
        let url = SiteInfo::parse_url(yaml)?;
        let description = SiteInfo::parse_description(yaml)?;
        let metadata = SiteInfo::parse_metadata(yaml)?;
        let default_timezone = SiteInfo::parse_default_timezone(yaml)?;

        Ok(SiteInfo {
               title: title,
               url: url,
               description: description,
               metadata: metadata,
               default_timezone: default_timezone,
           })
    }

    fn parse_title(yaml: &yaml::Hash) -> Result<String, String> {
        const TITLE: &'static str = "title";

        match yaml.get(&Yaml::from_str(TITLE)) {
            None |
            Some(&Yaml::Null) => Err(required_key(TITLE, yaml)),
            Some(&Yaml::String(ref string)) => Ok(string.clone()),
            _ => Err(key_of_type(TITLE, Required::Yes, yaml, "string")),
        }
    }

    fn parse_url(yaml: &yaml::Hash) -> Result<ValidatedUrl, String> {
        const URL: &'static str = "url";
        match yaml.get(&Yaml::from_str(URL)) {
            None |
            Some(&Yaml::Null) => Err(required_key(URL, yaml)),
            Some(&Yaml::String(ref string)) => ValidatedUrl::new(&string),
            _ => Err(key_of_type(URL, Required::Yes, yaml, "string")),
        }
    }

    fn parse_default_timezone(yaml: &yaml::Hash) -> Result<Tz, String> {
        unimplemented!()
    }

    fn parse_description(yaml: &yaml::Hash) -> Result<Option<String>, String> {
        const DESCRIPTION: &'static str = "description";
        match yaml.get(&Yaml::from_str(DESCRIPTION)) {
            None |
            Some(&Yaml::Null) => Ok(None),
            Some(&Yaml::String(ref string)) => Ok(Some(string.clone())),
            _ => Err(key_of_type(DESCRIPTION, Required::No, yaml, "string")),
        }
    }

    fn parse_metadata(yaml: &yaml::Hash) -> Result<HashMap<String, Yaml>, String> {
        const METADATA: &'static str = "metadata";
        let mut metadata = HashMap::new();
        match yaml.get(&Yaml::from_str(METADATA)) {
            None |
            Some(&Yaml::Null) => Ok(metadata),
            Some(&Yaml::Hash(ref hash)) => {
                for key in hash.keys() {
                    let key_str =
                        key.as_str()
                            .ok_or(key_of_type("key of hash map", Required::No, hash, "string"))?;

                    match hash.get(key) {
                        None |
                        Some(&Yaml::Null) => {
                            return Err(key_of_type(key_str, Required::No, hash, "hash"));
                        }
                        Some(inner_yaml @ &Yaml::String(..)) |
                        Some(inner_yaml @ &Yaml::Boolean(..)) |
                        Some(inner_yaml @ &Yaml::Integer(..)) |
                        Some(inner_yaml @ &Yaml::Real(..)) => {
                            let result = metadata.insert(String::from(key_str), inner_yaml.clone());
                            if result.is_some() {
                                let main = format!("Double insertion of key {}.\n", key_str);
                                let detail = format!("First: {:?}\nSecond: {:?}",
                                                     result.unwrap(),
                                                     inner_yaml);
                                return Err(main + &detail);
                            }
                        }
                        _ => {
                            return Err(key_of_type(key_str,
                                                   Required::No,
                                                   hash,
                                                   "string, boolean, or integer"))
                        }
                    }
                }
                Ok(metadata)
            }
            _ => Err(key_of_type(METADATA, Required::No, yaml, "hash")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_site_info(source: &str) -> BTreeMap<Yaml, Yaml> {
        let mut loaded = YamlLoader::load_from_str(source).unwrap();
        let first = loaded.pop().unwrap();
        first.as_hash().unwrap()[&Yaml::from_str("site_info")]
            .as_hash()
            .unwrap()
            .clone()
    }

    #[test]
    fn parses_title() {}

    #[test]
    fn parses_url() {}

    #[test]
    fn parses_metadata() {}

    #[test]
    fn parses_default_timezone() {}

    #[test]
    fn parses_site_info() {
        let site_info = "\
site_info:
    title: lx (lightning)
    url: https://lightning.rs
    description: >
        A ridiculously fast site generator and engine.
    default_timezone: Eastern
    metadata:
        foo: bar
        quux: 2
        ";

        let mut metadata = HashMap::new();
        metadata.insert("foo".into(), Yaml::from_str("bar"));
        metadata.insert("quux".into(), Yaml::from_str("2"));
        let expected = SiteInfo {
            title: "lx (lightning)".into(),
            url: ValidatedUrl::new("https://lightning.rs").unwrap(),
            description: Some("A ridiculously fast site generator and engine.\n".into()),
            default_timezone: UTC,
            metadata: metadata,
        };

        let site_info = load_site_info(site_info);
        assert_eq!(Ok(expected), SiteInfo::from_yaml(&site_info));
    }

    #[test]
    fn parses_site_info_with_empty_metadata() {
        let site_info_empty_metadata = "
site_info:
    title: lx (lightning)
    url: https://lightning.rs
    description: >
        A ridiculously fast site generator and engine.
    metadata: ~
        ";

        let expected = SiteInfo {
            title: "lx (lightning)".into(),
            url: ValidatedUrl::new("https://lightning.rs").unwrap(),
            description: Some("A ridiculously fast site generator and engine.\n".into()),
            default_timezone: UTC,
            metadata: HashMap::new(),
        };

        let site_info = load_site_info(site_info_empty_metadata);
        assert_eq!(Ok(expected), SiteInfo::from_yaml(&site_info));
    }
}