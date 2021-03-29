// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct AuthConfig {
    domain: String,
    client_id: String,
}

#[derive(Deserialize, Debug)]
struct User {
    nickname: String,
    name: String,
    picture: String,
    updated_at: String,
    email: String,
    email_verified: bool,
    sub: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn init_auth(domain: String, client_id: String) -> Result<JsValue, JsValue>;
}

#[derive(Default)]
struct Model {
    user: Option<User>,
    auth_config: Option<AuthConfig>,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async {
        Msg::AuthConfigFetched(
            async {
                fetch("/auth_config.json")
                    .await?
                    .check_status()?
                    .json()
                    .await
            }
            .await,
        )
    });
    Default::default()
}

enum Msg {
    AuthConfigFetched(fetch::Result<AuthConfig>),
    AuthInitialized(Result<JsValue, JsValue>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::AuthConfigFetched(Ok(auth_config)) => {
            let domain = auth_config.domain.clone();
            let client_id = auth_config.client_id.clone();
            orders.perform_cmd(async { Msg::AuthInitialized(init_auth(domain, client_id).await) });
            model.auth_config = Some(auth_config);
        }
        Msg::AuthConfigFetched(Err(err)) => {
            error!("AuthConfig fetch failed!", err);
        }
        Msg::AuthInitialized(Ok(user)) => {
            if not(user.is_undefined()) {
                match serde_wasm_bindgen::from_value(user) {
                    Ok(user) => model.user = Some(user),
                    Err(error) => error!("User deserialization failed!", error),
                }
            }
        }
        Msg::AuthInitialized(Err(error)) => {
            error!("Auth initialization failed!", error);
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn view(model: &Model) -> Node<Msg> {
    div![
        model
            .auth_config
            .as_ref()
            .map(|c| { div![format!("{:?}", c)] }),
        model.user.as_ref().map(|u| { div![format!("{:?}", u)] })
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
