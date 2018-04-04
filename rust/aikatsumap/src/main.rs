extern crate env_logger;
extern crate kuchiki;
extern crate reqwest;
extern crate url;

#[macro_use]
extern crate log;

use kuchiki::traits::TendrilSink;

#[derive(Debug)]
struct Shop {
    name: String,
    address: String,
}

fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let program = args.next().unwrap();
    let arg = args.next();
    if arg.is_none() {
        eprintln!("Usage: {} <BASE_URL>", program);
        std::process::exit(1);
    }

    let base_url = url::Url::parse(&arg.unwrap()).expect("Failed to parse BASE_URL");
    let index_url = base_url.join("index.php").unwrap();
    info!("GET {}", index_url);
    let text = reqwest::get(index_url).unwrap().text().unwrap();
    let document = kuchiki::parse_html().one(text);
    let mut shops = Vec::new();
    for a in document.select(".maplist dl dd a[href]").unwrap() {
        let attrs = a.attributes.borrow();
        let link = attrs.get("href").unwrap();
        let u = base_url.join(link).unwrap();
        let pref = u.query_pairs().find(|&(ref k, _)| k == "pref").unwrap().1;
        fetch_shops(
            &mut shops,
            base_url
                .join(&format!("shoplist.php?pref={}", pref))
                .unwrap(),
        );
    }

    println!("店名,住所");
    for shop in shops {
        println!("{},{}", shop.name, shop.address);
    }
}

const WHITE_SPACES: [char; 3] = ['\t', '\n', ' '];

fn fetch_shops(shops: &mut Vec<Shop>, url: url::Url) {
    info!("GET {}", url);
    let text = reqwest::get(url).unwrap().text().unwrap();
    let document = kuchiki::parse_html().one(text);
    for table in document.select(".resultlist").unwrap() {
        let name = table
            .as_node()
            .select_first(".list-name + td")
            .unwrap()
            .text_contents()
            .replace(WHITE_SPACES.as_ref(), "");
        let address = table
            .as_node()
            .select_first(".list-address + td")
            .unwrap()
            .text_contents()
            .replace(WHITE_SPACES.as_ref(), "");
        shops.push(Shop { name, address });
    }
}
