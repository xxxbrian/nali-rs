use super::geodb::{GeoDB, GeoLocation};

pub struct FakeGeo {}

impl FakeGeo {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FakeGeo {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoDB for FakeGeo {
    fn lookup(&self, _ip: &str) -> Option<GeoLocation> {
        Some(GeoLocation {
            location: "Fake Location".to_string(),
        })
    }
}
