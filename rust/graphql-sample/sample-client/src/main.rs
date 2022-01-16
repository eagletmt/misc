use wasm_bindgen::JsCast as _;
use wasm_bindgen::UnwrapThrowExt as _;
use yew::prelude::*;

#[derive(graphql_client::GraphQLQuery)]
#[graphql(
    schema_path = "../schema.json",
    query_path = "src/graphql/user.graphql",
    response_derive = "Debug"
)]
struct GetUser;

#[derive(Debug)]
enum Msg {
    UserId(i64),
    UserName(Option<String>),
    Error(String),
}

#[derive(Debug)]
struct Model {
    user_id: Option<i64>,
    user_name: Option<String>,
    error_message: Option<String>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            user_id: None,
            user_name: None,
            error_message: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::UserId(id) => {
                let need_update = self.user_id != Some(id);
                self.user_id = Some(id);
                ctx.link().send_future(async move {
                    let variables = get_user::Variables { id };
                    let client = reqwest::Client::new();
                    let result = graphql_client::reqwest::post_graphql::<GetUser, _>(
                        &client,
                        "http://localhost:3000/graphql",
                        variables,
                    )
                    .await;
                    match result {
                        Ok(resp) => Msg::UserName(
                            resp.data
                                .expect_throw("GraphQL data is missing")
                                .user
                                .map(|u| u.name),
                        ),
                        Err(e) => Msg::Error(format!("{}", e)),
                    }
                });
                need_update
            }
            Self::Message::UserName(name) => {
                let need_update = self.user_name != name;
                self.user_name = name;
                need_update
            }
            Self::Message::Error(e) => {
                let need_update = self.error_message.as_ref() != Some(&e);
                self.error_message = Some(e);
                need_update
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name_view = self
            .user_name
            .as_ref()
            .map(|name| format!("Hello, {}", name))
            .unwrap_or_default();
        let error_view = self
            .error_message
            .as_ref()
            .map(|m| format!("ERROR: {}", m))
            .unwrap_or_default();
        html! {
            <>
                <input type="number" label="User ID" onchange={ctx.link().callback(on_change)}/>
                <p>{name_view}</p>
                <p>{error_view}</p>
            </>
        }
    }
}

fn on_change(evt: web_sys::Event) -> Msg {
    let input = evt
        .target()
        .expect_throw("Event#target is not defined")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect_throw("failed to cast into HtmlInputElement");
    let value = input.value_as_number();
    Msg::UserId(value as i64)
}

fn main() {
    let window = web_sys::window().expect_throw("window is undefined");
    let document = window
        .document()
        .expect_throw("window.document is undefined");
    let e = document
        .get_element_by_id("main")
        .expect_throw("failed to find #main element");
    yew::start_app_in_element::<Model>(e);
}
