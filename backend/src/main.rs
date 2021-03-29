use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use anyhow::Context;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let authority = "https://hatoo.auth0.com/";
    let jwks = reqwest::get(format!("{}.well-known/jwks.json", authority))
        .await?
        .json::<JWKS>()
        .await?;

    let validations = vec![
        Validation::Issuer(authority.to_string()),
        Validation::SubjectPresent,
    ];

    let token = std::env::var("Token")?;

    let kid = token_kid(&token)
        .ok()
        .context("token_kid")?
        .context("token_kid")?;

    let jwk = jwks.find(&kid).context("find jwk")?;

    let res = validate(token.as_str(), jwk, validations)
        .ok()
        .context("validate")?;

    dbg!(res.claims);

    Ok(())
}
