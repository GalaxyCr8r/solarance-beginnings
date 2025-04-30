use anyhow::anyhow;
use openidconnect::{
    AccessTokenHash,
    AuthenticationFlow,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    IssuerUrl,
    Nonce,
    OAuth2TokenResponse,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenResponse,
};
use openidconnect::core::{
  CoreAuthenticationFlow,
  CoreClient,
  CoreProviderMetadata,
  CoreResponseType,
  CoreUserInfoClaims,
};
use openidconnect::reqwest;
use url::Url;
use dotenv::dotenv;
use std::env;

pub(crate) fn begin_connection() -> Result<(), String> {
    dotenv().ok();
    env_logger::init();
    
    let auth0_client_id = ClientId::new(
        env::var("AUTH0_CLIENT_ID").expect("Missing the AUTH0_CLIENT_ID environment variable."),
    );
    let auth0_client_secret: ClientSecret = ClientSecret::new(
        env::var("AUTH0_CLIENT_SECRET")
            .expect("Missing the AUTH0_CLIENT_SECRET environment variable."),
    );
    let issuer_url = IssuerUrl::new(env::var("AUTH0_ISSUER_URL").expect("Missing AUTH0_ISSUER_URL!")).expect("Invalid issuer URL");
    
    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover(
        &issuer_url,
        &http_client,
        ).unwrap();

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client =
    CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new("BnJiVrOXavZ1mbvsiwvBcZ96dTFH9k4L".to_string()),
        Some(ClientSecret::new("eJnzhmxpt_JeGfn6LxAwrypfPMlXzuhkfVKKS0exNvjsoQMQ6q2vEU6vAP0OSqeQ".to_string())),
    )
    // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(RedirectUrl::new("http://karlnyborg.com".to_string()).unwrap());

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
    .authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    )
    // Set the desired scopes.
    .add_scope(Scope::new("read".to_string()))
    .add_scope(Scope::new("write".to_string()))
    // Set the PKCE code challenge.
    .set_pkce_challenge(pkce_challenge)
    .url();

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    Ok(())
}

// fn finish_exchange() {
//     // Once the user has been redirected to the redirect URL, you'll have access to the
//     // authorization code. For security reasons, your code should verify that the `state`
//     // parameter returned by the server matches `csrf_state`.

//     // Now you can exchange it for an access token and ID token.
//     let token_response =
//     client
//         .exchange_code(AuthorizationCode::new("some authorization code".to_string()))?
//         // Set the PKCE code verifier.
//         .set_pkce_verifier(pkce_verifier)
//         .request(&http_client)?;

//     // Extract the ID token claims after verifying its authenticity and nonce.
//     let id_token = token_response
//     .id_token()
//     .ok_or_else(|| anyhow!("Server did not return an ID token"))?;
//     let id_token_verifier = client.id_token_verifier();
//     let claims = id_token.claims(&id_token_verifier, &nonce)?;

//     // Verify the access token hash to ensure that the access token hasn't been substituted for
//     // another user's.
//     if let Some(expected_access_token_hash) = claims.access_token_hash() {
//     let actual_access_token_hash = AccessTokenHash::from_token(
//         token_response.access_token(),
//         id_token.signing_alg()?,
//         id_token.signing_key(&id_token_verifier)?,
//     )?;
//     if actual_access_token_hash != *expected_access_token_hash {
//         return Err(anyhow!("Invalid access token"));
//     }
//     }

//     // The authenticated user's identity is now available. See the IdTokenClaims struct for a
//     // complete listing of the available claims.
//     println!(
//     "User {} with e-mail address {} has authenticated successfully",
//     claims.subject().as_str(),
//     claims.email().map(|email| email.as_str()).unwrap_or("<not provided>"),
//     );

//     // If available, we can use the user info endpoint to request additional information.

//     // The user_info request uses the AccessToken returned in the token response. To parse custom
//     // claims, use UserInfoClaims directly (with the desired type parameters) rather than using the
//     // CoreUserInfoClaims type alias.
//     let userinfo: CoreUserInfoClaims = client
//     .user_info(token_response.access_token().to_owned(), None)?
//     .request(&http_client)
//     .map_err(|err| anyhow!("Failed requesting user info: {}", err))?;

//     // See the OAuth2TokenResponse trait for a listing of other available fields such as
//     // access_token() and refresh_token().
//     Ok(())
// }