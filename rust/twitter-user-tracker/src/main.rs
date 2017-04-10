extern crate base64;
extern crate env_logger;
extern crate hyper;
extern crate hyper_native_tls;
extern crate postgres;
extern crate serde_json;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::io::{Read, Write};

#[derive(Deserialize, Debug)]
struct AccessToken {
    token_type: String,
    access_token: String,
}

#[derive(Debug)]
struct Tracker {
    client: hyper::Client,
    authorization_header: hyper::header::Authorization<hyper::header::Bearer>,
    postgres: postgres::Connection,
}

fn main() {
    env_logger::init().unwrap();

    let consumer_key = std::env::var("TWITTER_CONSUMER_KEY").expect("Set $TWITTER_CONSUMER_KEY");
    let consumer_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("Set $TWITTER_CONSUMER_SECRET");
    let postgres_url = std::env::var("POSTGRES_URL").expect("Set $POSTGRES_URL");

    let tls = hyper_native_tls::NativeTlsClient::new().unwrap();
    let client = hyper::Client::with_connector(hyper::net::HttpsConnector::new(tls));
    let response = client
        .post("https://api.twitter.com/oauth2/token")
        .body("grant_type=client_credentials")
        .header(hyper::header::Authorization(hyper::header::Basic {
                                                 username: consumer_key.to_owned(),
                                                 password: Some(consumer_secret.to_owned()),
                                             }))
        .header(hyper::header::ContentType::form_url_encoded())
        .send()
        .unwrap();
    match response.status {
        hyper::Ok => {
            let token: AccessToken = serde_json::from_reader(response).unwrap();

            let conn = postgres::Connection::connect(postgres_url, postgres::TlsMode::None)
                .unwrap();
            let tracker = Tracker::new(client, token.access_token, conn);
            for user_id in std::env::args().skip(1) {
                tracker.track_users(&user_id);
            }
        }
        _ => {
            writeln!(&mut std::io::stderr(), "Unable to get token").unwrap();
            die(response);
        }
    }
}

fn die(mut response: hyper::client::Response) {
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    writeln!(&mut std::io::stderr(), "{}", body).unwrap();
    std::process::exit(1);
}

#[derive(Deserialize, Debug)]
struct IdsResult {
    ids: Vec<i64>,
    next_cursor_str: String,
}

#[derive(Deserialize, Debug)]
struct UserResult {
    id: i64,
    screen_name: String,
}

impl Tracker {
    fn new(client: hyper::Client, access_token: String, postgres: postgres::Connection) -> Self {
        Self {
            client: client,
            authorization_header: hyper::header::Authorization(hyper::header::Bearer {
                                                                   token: access_token,
                                                               }),
            postgres: postgres,
        }
    }

    fn track_users(&self, user_id: &str) {
        self.track_users_1("followers", user_id);
        self.track_users_1("friends", user_id);
    }

    fn track_users_1(&self, api_name: &str, user_id: &str) {
        let mut cursor = "-1".to_owned();

        while cursor != "0" {
            let response = self.client
                .get(&format!("https://api.twitter.com/1.1/{}/ids.json?cursor={}&user_id={}&count=5000",
                              api_name, cursor, user_id))
                .header(self.authorization_header.clone())
                .send()
                .unwrap();
            match response.status {
                hyper::Ok => {
                    let result: IdsResult = serde_json::from_reader(response).unwrap();
                    info!("{}: Cursor is updated from {} to {}",
                          api_name,
                          cursor,
                          result.next_cursor_str);
                    cursor = result.next_cursor_str;
                    self.store_users(&result.ids);
                }
                _ => {
                    writeln!(&mut std::io::stderr(), "Unable to get {} ids", api_name).unwrap();
                    die(response);
                }
            }
        }
    }

    fn store_users(&self, user_ids: &[i64]) {
        for ids in user_ids.chunks(100) {
            let mut body = "user_id=".to_owned();
            for id in ids {
                body.push_str(&format!(",{}", id));
            }
            let response = self.client
                .post("https://api.twitter.com/1.1/users/lookup.json")
                .body(&body)
                .header(self.authorization_header.clone())
                .header(hyper::header::ContentType::form_url_encoded())
                .send()
                .unwrap();
            match response.status {
                hyper::Ok => {
                    let result: Vec<UserResult> = serde_json::from_reader(response).unwrap();
                    let mut txn = self.postgres.transaction().unwrap();
                    for user in result {
                        txn = self.store_user(txn, user);
                    }
                    txn.commit().unwrap();
                }
                _ => {
                    writeln!(&mut std::io::stderr(), "Unable to lookup users").unwrap();
                    die(response);
                }
            }
        }
    }

    fn store_user<'a>(&self,
                      txn: postgres::transaction::Transaction<'a>,
                      user: UserResult)
                      -> postgres::transaction::Transaction<'a> {
        txn.execute("insert into users (id, name) values ($1, $2) on conflict do nothing",
                     &[&user.id, &user.screen_name])
            .unwrap();
        txn
    }
}
