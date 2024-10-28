#[derive(Debug, Clone, PartialEq)]
pub struct GeoLocation {
    pub location: String,
}
pub trait GeoDB {
    fn lookup(&self, ip: &str) -> Option<GeoLocation>;
}
