use axum::{
    body::{Body, Bytes},
    extract::Request,
    middleware::Next,
    response::Response,
};
use axum_extra::{TypedHeader, headers::UserAgent, typed_header::TypedHeaderRejection};
use futures_util::StreamExt;

pub async fn curl_newline(
    user_agent: Result<TypedHeader<UserAgent>, TypedHeaderRejection>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    if user_agent.is_ok_and(|user_agent| user_agent.as_str().starts_with("curl/"))
        && response
            .headers()
            .get("content-type")
            .is_some_and(|ty| ty == "application/json" || ty == "text/plain; charset=utf-8")
    {
        if let Some(content_length) = response.headers_mut().remove("content-length")
            && let Ok(content_length) = content_length.to_str()
            && let Ok(content_length) = content_length.parse::<u64>()
            && let Some(content_length) = content_length.checked_add(2)
        {
            response
                .headers_mut()
                .insert("content-length", content_length.into());
        }
        let body = std::mem::take(response.body_mut());
        *response.body_mut() =
            Body::from_stream(body.into_data_stream().chain(futures_util::stream::once(
                futures_util::future::ready(Ok(Bytes::from_static(b"\n\r"))),
            )));
    }
    response
}
