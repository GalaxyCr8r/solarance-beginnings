use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::reqwest;
use openidconnect::{
    AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
};
use std::env;
use url::Url;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

pub fn get_client_token() -> Result<String, String> {
    let auth0_client_id = ClientId::new(
        env::var("AUTH0_CLIENT_ID").expect("Missing the AUTH0_CLIENT_ID environment variable."),
    );
    let issuer_url =
        IssuerUrl::new(env::var("AUTH0_ISSUER_URL").expect("Missing AUTH0_ISSUER_URL!"))
            .expect("Invalid issuer URL");

    let http_client = reqwest::blocking::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Use OpenID Connect Discovery to fetch the provider metadata.
    let provider_metadata = CoreProviderMetadata::discover(&issuer_url, &http_client).unwrap();

    // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
    // and token URL.
    let client = CoreClient::from_provider_metadata(provider_metadata, auth0_client_id, None)
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(RedirectUrl::new("http://localhost:13613".to_string()).unwrap());

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_state, _nonce) = client
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
    let _ = open::that_in_background(auth_url.to_string());

    let (code, state) = {
        // A very naive implementation of the redirect server.
        let listener = TcpListener::bind("127.0.0.1:13613").unwrap();

        // Accept one connection
        let (mut stream, _) = listener.accept().unwrap();

        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, code)| AuthorizationCode::new(code.into_owned()))
            .unwrap();

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, state)| CsrfToken::new(state.into_owned()))
            .unwrap();

        let message = {
            let mut tmp = "<h1>Login Completed!</h1>".to_string();
            tmp += "<p>Return to <em>Solarance:Beginnings</em> and complain about this travesty of a landing page!</p>";
            tmp = format!("<center>{}</center>", tmp);
            format!("<html><body>{}</body></html>", tmp)
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        stream.write_all(response.as_bytes()).unwrap();

        (code, state)
    };

    println!("Auth0 returned the following code:\n{}\n", code.secret());
    println!(
        "Auth0 returned the following state:\n{} (expected `{}`)\n",
        state.secret(),
        csrf_state.secret()
    );

    // Exchange the code with a token.
    let token_response = client
        .exchange_code(code)
        .expect("No user info endpoint")
        .set_pkce_verifier(pkce_verifier)
        .request(&http_client)
        .expect("Failed to contact token endpoint");

    println!(
        "Auth0 returned access token:\n{}\n",
        token_response.access_token().secret()
    );
    println!("Auth0 returned scopes: {:?}", token_response.scopes());

    Ok(token_response
        .id_token()
        .expect("ID token is missing or malformed.")
        .to_string())
}
