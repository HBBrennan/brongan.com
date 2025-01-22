use crate::ServerState;
use axum::{
    body::BoxBody,
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use hyper::{header::ACCEPT, HeaderMap};
use opentelemetry::{
    global,
    trace::{get_active_span, FutureExt, TraceContextExt, Tracer},
    Context, KeyValue,
};
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use tracing::{info, warn};

fn get_fly_client_addr(headers: &HeaderMap) -> Option<IpAddr> {
    let header = headers.get("fly-client-ip")?;
    let header = header.to_str().ok()?;
    let addr = header.parse::<IpAddr>().ok()?;
    Some(addr)
}

pub async fn analytics_get(
    headers: HeaderMap,
    State(state): State<ServerState>,
) -> Response<BoxBody> {
    // Print something to the console
    let tracer = global::tracer("");
    let span = tracer.start("analytics_get");
    let analytics = state
        .locat
        .get_analytics()
        .with_context(Context::current_with_span(span))
        .await
        .unwrap();
    if let Some(content_type) = headers.get(ACCEPT) {
        if content_type == "*/*" {
            return serde_json::to_string(&analytics).unwrap().into_response();
        }
    }
    analytics
        .iter()
        .map(|analytics| analytics.to_string())
        .collect::<Vec<String>>()
        .join("\n")
        .into_response()
}

pub async fn record_analytics<B>(
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    if request.uri().path() == "/_trunk/ws" {
        return next.run(request).await;
    }
    let path = Path::new(request.uri().path());

    if let Some("wasm" | "js" | "png" | "jpg" | "vert" | "scss" | "frag" | "css") =
        path.extension().and_then(|os_str| os_str.to_str())
    {
        return next.run(request).await;
    }

    let ip = get_fly_client_addr(&headers).unwrap_or(addr.ip());
    let iso_code = if ip.is_loopback() {
        "DEV"
    } else {
        let iso_code = state.locat.get_iso_code(ip).await;
        match &iso_code {
            Ok(country) => {
                info!("Received request from {country}");
                get_active_span(|span| {
                    span.set_attribute(KeyValue::new("country", country.to_string()))
                });
                country
            }
            Err(err) => {
                warn!("Could not determine country for IP {addr}: {err}");
                "N/A"
            }
        }
    };

    match state
        .locat
        .record_request(ip, iso_code.to_owned(), request.uri().path().to_owned())
        .await
    {
        Ok(_) => info!("Recorded request from {ip} for {}", request.uri().path()),
        Err(err) => warn!("Failed to record request from {ip}: {}", err),
    }

    next.run(request).await
}
