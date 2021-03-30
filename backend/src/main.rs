use alcoholic_jwt::{token_kid, validate, ValidJWT, Validation, JWKS};
use anyhow::Context;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let authority = std::env::var("Authority")?;
    let client = reqwest::Client::default();

    let auth = warp::header::<String>("Authorization")
        .or(warp::any().map(|| String::new()))
        .unify()
        .and(warp::any().map(move || authority.clone()))
        .and(warp::any().map(move || client.clone()))
        .and_then(
            |bearer_token: String, authority: String, client: reqwest::Client| async move {
                Ok::<Option<ValidJWT>, std::convert::Infallible>(
                    match bearer_token
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .as_slice()
                    {
                        &["Bearer", token] => validate_token(&client, authority.as_str(), token)
                            .await
                            .ok(),
                        _ => None,
                    },
                )
            },
        );

    let api = warp::path("api").and(auth).map(|jwt: Option<ValidJWT>| {
        jwt.map(|jwt| format!("{:?}", jwt.claims))
            .unwrap_or("Authorization Failed".to_string())
    });

    warp::serve(
        api.with(
            warp::cors()
                .allow_any_origin()
                .allow_methods(&[warp::http::Method::GET])
                .allow_headers(&[warp::http::header::AUTHORIZATION]),
        ),
    )
    .run(([127, 0, 0, 1], 3030))
    .await;
    Ok(())
}

async fn validate_token(
    client: &reqwest::Client,
    authority: &str,
    token: &str,
) -> Result<ValidJWT, anyhow::Error> {
    let jwks = client
        .get(format!("{}.well-known/jwks.json", authority))
        .send()
        .await?
        .json::<JWKS>()
        .await?;

    let validations = vec![
        Validation::Issuer(authority.to_string()),
        Validation::SubjectPresent,
        Validation::NotExpired,
    ];

    let kid = token_kid(&token)
        .ok()
        .context("token_kid")?
        .context("token_kid")?;

    let jwk = jwks.find(&kid).context("find jwk")?;

    let res = validate(token, jwk, validations).ok().context("validate")?;

    Ok(res)
}
