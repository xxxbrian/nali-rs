use crate::{geo::geodb::GeoDB, NaliText};

pub trait Parser<G: GeoDB> {
    fn parse(&self, input: &str, db: &G) -> NaliText;
    fn name(&self) -> &str;
}
