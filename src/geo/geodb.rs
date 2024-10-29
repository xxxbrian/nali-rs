#[derive(Debug, Clone, PartialEq)]
pub struct GeoLocation {
    pub location: String,
}
pub trait GeoDB {
    fn lookup(&self, ip: &str) -> Option<GeoLocation>;
}

impl GeoDB for Box<dyn GeoDB> {
    fn lookup(&self, ip: &str) -> Option<GeoLocation> {
        (**self).lookup(ip)
    }
}
