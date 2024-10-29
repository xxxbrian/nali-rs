use nali::{
    geo::{fakegeo::FakeGeo, geodb::GeoDB, geolite2::GeoLite2},
    FastParser, Parser, RegexParser,
};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let nali_config = nali::config::NaliConfig::new();

    let geo: Box<dyn GeoDB> = match nali_config.geodb() {
        nali::config::GeoDBConfig::FakeGeo(_) => Box::new(FakeGeo::new()),
        nali::config::GeoDBConfig::GeoLite2(geolite2_config) => Box::new(GeoLite2::new(
            &geolite2_config.full_path(nali_config.app_support_path()),
        )),
    };
    let parser: Box<dyn Parser<_>> = match nali_config.parser() {
        nali::config::ParserConfig::FastParser => Box::new(FastParser::default()),
        nali::config::ParserConfig::RegexParser => Box::new(RegexParser::default()),
    };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let nali_text = parser.parse(&line, &geo);
        writeln!(stdout, "{}", nali_text.colorize())?;
        stdout.flush()?;
    }

    Ok(())
}
