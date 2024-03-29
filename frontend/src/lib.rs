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

    #[wasm_bindgen(catch)]
    async fn get_token() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn redirect_to_sign_up() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn redirect_to_log_in() -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    fn logout() -> Result<(), JsValue>;
}

#[derive(Default)]
struct Model {
    base_url: Url,
    user: Option<User>,
    auth_config: Option<AuthConfig>,
    token: Option<String>,
    api_response: Option<String>,
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
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
    Model {
        base_url: url,
        ..Default::default()
    }
}

enum Msg {
    AuthConfigFetched(fetch::Result<AuthConfig>),
    AuthInitialized(Result<JsValue, JsValue>),
    SignUp,
    LogIn,
    LogOut,
    RedirectingToSignUp(Result<(), JsValue>),
    RedirectingToLogIn(Result<(), JsValue>),
    ShowToken,
    TokenFetched(Result<JsValue, JsValue>),
    CallAPI,
    APIfetched(fetch::Result<String>),
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
            let search = model.base_url.search_mut();
            if search.remove("code").is_some() && search.remove("state").is_some() {
                model.base_url.go_and_replace();
            }
        }
        Msg::AuthInitialized(Err(error)) => {
            error!("Auth initialization failed!", error);
        }
        Msg::SignUp => {
            orders.perform_cmd(async { Msg::RedirectingToSignUp(redirect_to_sign_up().await) });
        }
        Msg::LogIn => {
            orders.perform_cmd(async { Msg::RedirectingToLogIn(redirect_to_log_in().await) });
        }
        Msg::RedirectingToSignUp(result) => {
            if let Err(error) = result {
                error!("Redirect to sign up failed!", error);
            }
        }
        Msg::RedirectingToLogIn(result) => {
            if let Err(error) = result {
                error!("Redirect to log in failed!", error);
            }
        }
        Msg::LogOut => {
            if let Err(error) = logout() {
                error!("Cannot log out!", error);
            } else {
                model.user = None;
            }
        }
        Msg::ShowToken => {
            orders.perform_cmd(async { Msg::TokenFetched(get_token().await) });
        }

        Msg::TokenFetched(Ok(token)) => {
            model.token = token.as_string();
        }

        Msg::TokenFetched(Err(error)) => {
            error!("Cannot get token!", error);
        }

        Msg::CallAPI => {
            orders.perform_cmd(async {
                Msg::APIfetched(
                    async {
                        let token = get_token()
                            .await
                            .ok()
                            .and_then(|js_value| js_value.as_string())
                            .unwrap_or_default();

                        fetch::Request::new("http://localhost:3030/api")
                            .header(fetch::Header::authorization(format!("Bearer {}", token)))
                            .fetch()
                            .await?
                            .check_status()?
                            .text()
                            .await
                    }
                    .await,
                )
            });
        }
        Msg::APIfetched(Ok(response)) => {
            model.api_response = Some(response);
        }
        Msg::APIfetched(Err(error)) => {
            error!("API call failed!", error);
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn view(model: &Model) -> Node<Msg> {
    div![
        p![model
            .auth_config
            .as_ref()
            .map(|c| { div![format!("{:?}", c)] })],
        p![model.user.as_ref().map(|u| { div![format!("{:?}", u)] })],
        p![model.token.as_ref()],
        button!["Sign up", ev(Ev::Click, |_| Msg::SignUp),],
        button!["Log in", ev(Ev::Click, |_| Msg::LogIn),],
        button!["Log out", ev(Ev::Click, |_| Msg::LogOut),],
        button!["get token", ev(Ev::Click, |_| Msg::ShowToken),],
        button!["Call API", ev(Ev::Click, |_| Msg::CallAPI)],
        p![model.api_response.as_ref()]
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
