use crate::auth::{account::MutationRoot, db::get_storage};
use actix_cors::Cors;
use actix_web::{
    dev::ServiceRequest, guard, http::header::HeaderMap, web, App, Error, HttpRequest,
    HttpResponse, HttpServer, Result,
};
use actix_web_httpauth::{
    extractors::{basic::BasicAuth, bearer::BearerAuth, AuthenticationError},
    middleware::HttpAuthentication,
};
use async_graphql::{http::GraphiQLSource, Data, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use auth::account::{AccountSchema, QueryRoot, SubscriptionRoot};
use config::Config;
use log::info;

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
        let auth = HttpAuthentication::basic(ok_validator);
        App::new()
            .wrap(auth)
            // ensure the CORS middleware is wrapped around the httpauth middleware so it is able to
            // add headers to error responses
            .wrap(Cors::permissive())
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

async fn ok_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    eprintln!("{:?}", credentials);
    let http_user = std::env::var("HTTP_AUTH_USER").unwrap_or("".to_string());
    let http_pass = std::env::var("HTTP_AUTH_PASSWORD").unwrap_or("".to_string());
    if credentials.user_id().eq(http_user.as_str()) && credentials.password().eq(&Some(http_pass.as_str())) {
        Ok(req)
    } else {
        use actix_web_httpauth::headers::www_authenticate::basic::Basic;
        Err((Error::from(AuthenticationError::new(Basic::new())), req))
    }
}
