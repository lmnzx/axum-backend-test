use axum::{
    body::Bytes,
    extract::MatchedPath,
    http::Request,
    response::{Html, Response},
    routing::get,
    Router,
};
use tokio::{net::TcpListener, time::Duration};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{info_span, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn hello() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "test-app=trace,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/", get(hello)).layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            })
            .on_request(|request: &Request<_>, _span: &Span| {
                tracing::debug!("started {} {}", request.method(), request.uri().path())
            })
            .on_response(|_response: &Response, latency: Duration, _span: &Span| {
                tracing::debug!("response generated in {:?}", latency)
            })
            .on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
                tracing::debug!("sending {} bytes", chunk.len())
            })
            .on_failure(
                |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    tracing::error!("something went wrong {:#?}", error)
                },
            ),
    );

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
