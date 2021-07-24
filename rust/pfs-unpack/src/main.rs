/*
 * This code is based on GARbro written by morkt under MIT license.
 * https://github.com/morkt/GARbro
 */

use anyhow::Context as _;
use byteorder::ReadBytesExt as _;
use sha1::Digest as _;
use std::io::Read as _;
use std::io::Seek as _;
use std::io::Write as _;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    for arg in std::env::args().skip(1) {
        unpack(&arg).with_context(|| format!("failed to unpack {}", arg))?;
    }
    Ok(())
}

#[derive(Debug)]
struct Entry {
    name: String,
    offset: u32,
    size: u32,
}

fn unpack<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    let mut magic = [0; 2];
    reader.read_exact(&mut magic)?;
    if &magic != b"pf" {
        return Err(anyhow::anyhow!("not an Artemis archive"));
    }
    let mut version = [0; 1];
    reader.read_exact(&mut version)?;
    let version = version[0] - b'0';
    if version != 8 {
        return Err(anyhow::anyhow!("unknown version number: {}", version));
    }

    let index_size = reader.read_u32::<byteorder::LittleEndian>()? as usize;
    log::debug!("index_size = {}", index_size);

    let pos = reader.stream_position()?;
    let mut index = vec![0u8; index_size];
    reader.read_exact(&mut index)?;
    let key = sha1::Sha1::digest(&index);
    reader.seek(std::io::SeekFrom::Start(pos))?;

    let file_count = reader.read_u32::<byteorder::LittleEndian>()?;
    log::debug!("file_count = {}", file_count);
    let mut entries = Vec::new();
    for i in 0..file_count {
        let name_length = reader.read_u32::<byteorder::LittleEndian>()? as usize;
        log::debug!("[{}] name_length={}", i, name_length);
        let mut name = vec![0u8; name_length];
        reader.read_exact(&mut name)?;
        let name = String::from_utf8(name)?;
        log::debug!("  name={}", name);
        reader.read_u32::<byteorder::LittleEndian>()?; // Skip 4 bytes
        let offset = reader.read_u32::<byteorder::LittleEndian>()?;
        let size = reader.read_u32::<byteorder::LittleEndian>()?;
        log::debug!("  offset={} size={}", offset, size);
        entries.push(Entry { name, offset, size });
    }

    let separator = std::path::MAIN_SEPARATOR.to_string();
    let mut buf = vec![0u8; 16 * 1024];
    let buf_size = buf.len();
    for entry in entries {
        reader.seek(std::io::SeekFrom::Start(entry.offset as u64))?;
        let p = std::path::PathBuf::from(entry.name.replace('\\', &separator));
        log::info!("Unpack {} (size={})", p.display(), entry.size);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(&p)?;
        let mut writer = std::io::BufWriter::new(file);
        let mut read_bytes = 0;
        let size = entry.size as usize;
        while read_bytes < size {
            let r = reader.read(&mut buf[0..std::cmp::min(buf_size, size - read_bytes)])?;
            for i in 0..r {
                buf[i] ^= key[(read_bytes + i) % key.len()];
            }
            read_bytes += r;
            writer.write_all(&buf[0..r])?;
        }
    }

    Ok(())
}
