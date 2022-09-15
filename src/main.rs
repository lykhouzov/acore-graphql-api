use actix_web::{
    guard, http::header::HeaderMap, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::{http::GraphiQLSource, Data, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use auth::account::{AccountSchema, QueryRoot, SubscriptionRoot};
use log::info;

use crate::auth::{account::MutationRoot, db::get_storage};

mod auth;

async fn graphiql() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint("http://localhost:8000")
                .subscription_endpoint("ws://localhost:8000/ws")
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
const DEFAULT_HOST: &'static str = "127.0.0.1";
const DEFAUTL_PORT: u16 = 8000;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv::dotenv().ok();
    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let server_port = std::env::var("SERVER_PORT").map_or_else(
        |_| DEFAUTL_PORT,
        |v| v.parse::<u16>().unwrap_or(DEFAUTL_PORT),
    );
    env_logger::init();
    let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(get_storage().await)
        .finish();

    info!("GraphiQL IDE: http://{}:{}", server_host, server_port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Get()).to(graphiql))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/ws").to(index_ws))
    })
    .bind((server_host, server_port))?
    .run()
    .await
}
