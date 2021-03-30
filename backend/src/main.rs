use alcoholic_jwt::{token_kid, validate, ValidJWT, Validation, JWKS};
use anyhow::Context;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = reqwest::Client::default();
    let authority = "https://hatoo.auth0.com/";

    let res = validate_token(&client, authority, std::env::var("Token")?.as_str()).await?;

    dbg!(res.claims);

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
