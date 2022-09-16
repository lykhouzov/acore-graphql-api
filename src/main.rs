use actix_web::{
    guard, http::header::HeaderMap, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::{http::GraphiQLSource, Data, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use auth::account::{AccountSchema, QueryRoot, SubscriptionRoot};
use config::Config;
use log::info;

use crate::auth::{account::MutationRoot, db::get_storage};

mod auth;
pub mod config;

async fn graphiql(config: web::Data<Config>) -> HttpResponse {
    let endpoint = format!("http://{}:{}", config.host(), config.port());
    let endpoint_sub = format!("http://{}:{}/ws", config.host(), config.port());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint(endpoint.as_str())
                .subscription_endpoint(endpoint_sub.as_str())
                .finish(),
        )
}

fn get_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Token")
        .and_then(|value| value.to_str().map(|s| s.to_string()).ok())
}

async fn index(
    schema: web::Data<AccountSchema>,
    _req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let request = gql_request.into_inner();
    // if let Some(token) = get_token_from_headers(req.headers()) {
    //     request = request.data(token);
    // }
    schema.execute(request).await.into()
}

async fn index_ws(
    schema: web::Data<AccountSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    let mut data = Data::default();
    if let Some(token) = get_token_from_headers(req.headers()) {
        data.insert(token);
    }

    GraphQLSubscription::new(Schema::clone(&*schema))
        .with_data(data)
        // .on_connection_init(on_connection_init)
        .start(&req, payload)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv::dotenv().ok();
    // let config = Config::from_env();
    let config = web::Data::new(Config::from_env());
    env_logger::init();
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(get_storage().await)
        .finish();

    info!("GraphiQL IDE: http://{}:{}", config.host(), config.port());
    let (server_host, server_port) = { (config.host(), config.port()) };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(config.clone())
            .service(web::resource("/").guard(guard::Get()).to(graphiql))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/ws").to(index_ws))
    })
    .bind((server_host, server_port))?
    .run()
    .await
}
