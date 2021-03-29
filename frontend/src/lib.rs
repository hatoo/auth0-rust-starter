// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
use serde::Deserialize;

#[derive(Deserialize)]
struct AuthConfig {
    domain: String,
    client_id: String,
}

struct Model {
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
    Model { auth_config: None }
}

enum Msg {
    AuthConfigFetched(fetch::Result<AuthConfig>),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::AuthConfigFetched(auth_config) => {
            if let Ok(auth_config) = auth_config {
                model.auth_config = Some(auth_config);
            }
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn view(model: &Model) -> Node<Msg> {
    div![model
        .auth_config
        .as_ref()
        .map(|c| { div![c.domain.as_str(), c.client_id.as_str()] })]
}

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
