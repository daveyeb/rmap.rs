use axum::{body::Body, http::Request, middleware::Next, response::Response};

pub async fn propagate_coop_coep_headers(
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let mut response = next.run(req).await;

    response.headers_mut().insert(
        "Cross-Origin-Embedder-Policy",
        "require-corp".parse().unwrap(),
    );
    response
        .headers_mut()
        .insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());

    Ok(response)
}
