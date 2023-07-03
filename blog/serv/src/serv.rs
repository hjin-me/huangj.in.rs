use crate::fallback::file_and_error_handler;
use axum::extract::{Path, RawQuery};
use axum::response::IntoResponse;
use axum::{
    body::Body as AxumBody,
    extract::Extension,
    http::{header::HeaderMap, Request},
    routing::{any, get},
    Router,
};
use biz::github_hook;
use clap::Parser;
use elasticsearch::Elasticsearch;
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use std::fs;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use ui::home::BlogApp;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of times to greet
    #[arg(short, long, default_value = "./config.toml")]
    config: String,
    #[arg(short, long, default_value = "info")]
    log: String,
}

pub async fn serv() {
    let args = Args::parse();
    // a builder for `FmtSubscriber`.
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(args.log.parse::<Level>().unwrap_or(Level::INFO))
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // get pwd
    let pwd = std::env::current_dir().unwrap();
    info!("Starting up {}, {:?}", &args.config, pwd);
    let contents =
        fs::read_to_string(&args.config).expect("Should have been able to read the file");
    let serv_conf: biz::Config = toml::from_str(contents.as_str()).unwrap();

    let es_client = Arc::new(biz::es::init(&serv_conf.es_url).expect("初始化ES失败"));

    biz::serv(&es_client, &serv_conf)
        .await
        .expect("同步文章数据失败");

    // Setting this to None means we'll be using cargo-leptos and its env vars
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! { cx, <BlogApp/> }).await;

    let leptos_es_client = es_client.clone();
    // build our application with a route
    let app = Router::new()
        .layer(CompressionLayer::new())
        .route("/liveness", get(|| async { "I'm alive!" }))
        .route("/readiness", get(|| async { "I'm ready!" }))
        .route("/hook/github", any(github_hook::github_hook))
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move |cx| {
                provide_context(cx, leptos_es_client.clone());
            },
            |cx| view! { cx, <BlogApp/> },
        )
        .fallback(file_and_error_handler)
        .with_state(leptos_options.clone())
        .layer(Extension(Arc::new(leptos_options)))
        .layer(Extension(Arc::new(serv_conf)))
        .layer(Extension(es_client))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new()),
        );

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn server_fn_handler(
    Extension(es_client): Extension<Arc<Elasticsearch>>,
    path: Path<String>,
    headers: HeaderMap,
    raw_query: RawQuery,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        path,
        headers,
        raw_query,
        move |cx| {
            provide_context(cx, es_client.clone());
        },
        request,
    )
    .await
}
