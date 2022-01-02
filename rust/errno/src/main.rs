include!(concat!(env!("OUT_DIR"), "/errno.rs"));

fn main() {
    for arg in std::env::args().skip(1) {
        if let Ok(n) = arg.parse::<i32>() {
            for (name, value, message) in ERROR_NUMBERS.iter().filter(|(_, value, _)| *value == n) {
                println!("{}={}: {}", name, value, message);
            }
        }

        for (name, value, message) in ERROR_NUMBERS
            .iter()
            .filter(|(name, _, message)| name.contains(&arg) || message.contains(&arg))
        {
            println!("{}={}: {}", name, value, message);
        }
    }
}
