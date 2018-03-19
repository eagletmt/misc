extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[derive(Debug, Serialize)]
struct DiscoveryData {
    data: Vec<DiscoveryEntry>,
}

#[derive(Debug, Serialize)]
struct DiscoveryEntry {
    #[serde(rename = "{#DEVICE_NAME}")] device_name: String,
    #[serde(rename = "{#SENSOR_NAME}")] sensor_name: String,
}

fn main() {
    use std::io::Read;

    let mut data = vec![];
    for entry in std::fs::read_dir("/sys/class/hwmon").expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let pathbuf = entry.path();
        if let Ok(mut file) = std::fs::File::open(pathbuf.join("name")) {
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .expect("Failed to read name file");
            let name = buf.trim();

            for entry in std::fs::read_dir(pathbuf).expect("Failed to read hwmon directory") {
                let entry = entry.expect("Failed to read entry");
                let path = entry.path();
                let filename = path.file_name().unwrap().to_string_lossy();
                const SUFFIX: &str = "_input";
                if filename.ends_with(SUFFIX)
                    && entry.file_type().expect("Failed to get filetype").is_file()
                {
                    data.push(DiscoveryEntry {
                        device_name: name.to_owned(),
                        sensor_name: filename[..filename.len() - SUFFIX.len()].to_owned(),
                    });
                }
            }
        }
    }
    serde_json::to_writer_pretty(std::io::stdout(), &DiscoveryData { data })
        .expect("Failed to write JSON");
    println!();
}
