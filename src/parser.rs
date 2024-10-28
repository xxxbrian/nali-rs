use crate::NaliText;

pub trait Parser {
    fn parse(&self, input: &str) -> NaliText;
    fn name(&self) -> &str;
}
