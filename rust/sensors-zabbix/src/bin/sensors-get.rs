fn main() {
    use std::io::Read;
    use std::io::Write;

    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        writeln!(
            std::io::stderr(),
            "Usage: {} <device_name> <sensor_name>",
            args[0]
        ).unwrap();
        std::process::exit(1);
    }
    let device_name = &args[1];
    let sensor_name = &args[2];

    for entry in std::fs::read_dir("/sys/class/hwmon").expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let pathbuf = entry.path();
        if let Ok(mut file) = std::fs::File::open(pathbuf.join("name")) {
            let mut buf = String::new();
            file.read_to_string(&mut buf).expect(
                "Failed to read name file",
            );
            let name = buf.trim();

            if name == device_name {
                if let Ok(mut file) = std::fs::File::open(pathbuf.join(sensor_name)) {
                    let mut buf = String::new();
                    file.read_to_string(&mut buf).expect(
                        "Failed to read sensor value",
                    );
                    let mut value = buf.trim().parse::<f64>().expect(
                        "Failed to parse sensor value",
                    );
                    if !is_fan(sensor_name) {
                        value /= 1000.0;
                    }
                    println!("{}", value);
                }
            }
        }
    }
}

fn is_fan(sensor_name: &str) -> bool {
    sensor_name.starts_with("fan")
}
