/*
 * Usage: cargo run
 * sudo cp UTF-8-CJK.gz /usr/share/i18n/charmaps/
 * sudo vim /etc/locale.gen  # Add "ja_JP.UTF-8 UTF-8-CJK"
 * sudo locale-gen
 */

const UTF8_CHARMAP: &str = "/usr/share/i18n/charmaps/UTF-8.gz";
const EAST_ASIAN_WIDTH_URL: &str = "https://www.unicode.org/Public/UNIDATA/EastAsianWidth.txt";

use std::io::{BufRead, Write};

#[derive(Debug)]
struct CharRange {
    start: u32,
    end: u32,
}
impl std::fmt::Display for CharRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.start == self.end {
            Ok(())
        } else if self.start + 1 == self.end {
            format_codepoint(f, self.start)?;
            write!(f, "\t\t\t2")
        } else {
            format_codepoint(f, self.start)?;
            write!(f, "...")?;
            format_codepoint(f, self.end - 1)?;
            write!(f, "\t2")
        }
    }
}

fn format_codepoint(f: &mut std::fmt::Formatter, c: u32) -> std::fmt::Result {
    if c < 0x10000 {
        write!(f, "<U{:04X}>", c)
    } else {
        write!(f, "<U{:08X}>", c)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(EAST_ASIAN_WIDTH_URL)
        .header(
            reqwest::header::ACCEPT_ENCODING,
            reqwest::header::HeaderValue::from_static("gzip"),
        )
        .send()
        .await?
        .error_for_status()?;
    let body = resp.text().await?;

    let single_re = regex_lite::Regex::new(r"\A([0-9A-F]+);A")?;
    let multi_re = regex_lite::Regex::new(r"\A([0-9A-F]+)\.\.([0-9A-F]+);A")?;

    let mut ranges = Vec::new();
    let mut cr = CharRange { start: 0, end: 0 };
    for line in body.lines() {
        if let Some(caps) = single_re.captures(line) {
            let c = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16)?;
            if cr.end == c {
                cr.end += 1;
            } else {
                ranges.push(cr);
                cr = CharRange {
                    start: c,
                    end: c + 1,
                };
            }
        } else if let Some(caps) = multi_re.captures(line) {
            let c1 = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16)?;
            let c2 = u32::from_str_radix(caps.get(2).unwrap().as_str(), 16)?;
            if cr.end == c1 {
                cr.end += c2 - c1 + 1;
            } else {
                ranges.push(cr);
                cr = CharRange {
                    start: c1,
                    end: c2 + 1,
                };
            }
        }
    }
    ranges.push(cr);

    let mut writer = std::io::BufWriter::new(flate2::write::GzEncoder::new(
        std::fs::File::create("UTF-8-CJK.gz")?,
        flate2::Compression::default(),
    ));
    let reader = std::io::BufReader::new(flate2::bufread::GzDecoder::new(std::io::BufReader::new(
        std::fs::File::open(UTF8_CHARMAP)?,
    )));
    for line in reader.lines() {
        let line = line?;
        if line == "END WIDTH" {
            for cr in ranges.drain(..) {
                writeln!(writer, "{}", cr)?;
            }
        }
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}
