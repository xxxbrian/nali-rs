use std::collections::BTreeMap;

use maxminddb::geoip2;
use sys_locale::get_locale;

use super::geodb::{GeoDB, GeoLocation};

pub struct GeoLite2 {
    reader: maxminddb::Reader<Vec<u8>>,
    language: String,
}

impl GeoLite2 {
    pub fn new(mmdb_path: &str) -> Self {
        let reader = maxminddb::Reader::open_readfile(mmdb_path).unwrap();
        let l_code = if let Some(locale) = get_locale() {
            match locale.as_str() {
                l if l.starts_with("de") => "de",
                l if l.starts_with("en") => "en",
                l if l.starts_with("es") => "es",
                l if l.starts_with("fr") => "fr",
                l if l.starts_with("ja") => "ja",
                l if l.starts_with("pt") => "pt",
                l if l.starts_with("ru") => "ru",
                l if l.starts_with("zh") => "zh-CN",
                _ => "en",
            }
        } else {
            "en" // Default to English if locale is unavailable
        };
        Self {
            reader,
            language: String::from(l_code),
        }
    }

    fn get_localized_name(names: &Option<BTreeMap<&str, &str>>, language: &str) -> Option<String> {
        names.as_ref().and_then(|names| {
            names
                .get(language)
                .map(|s| s.to_string())
                .or_else(|| names.get("en").map(|s| s.to_string()))
        })
    }
}

impl GeoDB for GeoLite2 {
    fn lookup(&self, ip: &str) -> Option<GeoLocation> {
        let ip = ip.parse().unwrap();
        let city_data: geoip2::City = self.reader.lookup(ip).unwrap_or(geoip2::City {
            city: None,
            continent: None,
            country: None,
            location: None,
            postal: None,
            registered_country: None,
            represented_country: None,
            subdivisions: None,
            traits: None,
        });
        let city_name =
            Self::get_localized_name(&city_data.city.and_then(|city| city.names), &self.language);
        let country_name = Self::get_localized_name(
            &city_data.country.and_then(|country| country.names),
            &self.language,
        );
        match (city_name, country_name) {
            (Some(city), Some(country)) => Some(GeoLocation {
                location: format!("{} {}", country, city),
            }),
            (Some(city), None) => Some(GeoLocation { location: city }),
            (None, Some(country)) => Some(GeoLocation { location: country }),
            (None, None) => None,
        }
    }
}
