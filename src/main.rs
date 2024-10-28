use nali::{geo::geolite2::GeoLite2, Parser, RegexParser};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let app_support_path = dirs::config_dir().unwrap().join("nali-rs");
    let parser = RegexParser::new(GeoLite2::new(
        app_support_path
            .join("GeoLite2-City.mmdb")
            .to_str()
            .unwrap(),
    ));
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let nali_text = parser.parse(&line);
        writeln!(stdout, "{}", nali_text.colorize())?;
        stdout.flush()?;
    }

    Ok(())
}
