#[derive(Debug, juniper::GraphQLObject, sqlx::FromRow)]
#[graphql()]
struct User {
    id: i32,
    name: String,
}

#[derive(Clone)]
struct Context {
    pool: sqlx::PgPool,
}
impl juniper::Context for Context {}

struct Query;
#[juniper::graphql_object(context = Context)]
impl Query {
    async fn user(context: &Context, id: i32) -> juniper::FieldResult<Option<User>> {
        let user: Option<User> = sqlx::query_as("select id, name from users where id = $1")
            .bind(id)
            .fetch_optional(&context.pool)
            .await?;
        Ok(user)
    }
}

type Schema = juniper::RootNode<
    'static,
    Query,
    juniper::EmptyMutation<Context>,
    juniper::EmptySubscription<Context>,
>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,tower_http::trace=debug");
    }
    tracing_subscriber::fmt::init();

    let pool = sqlx::PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("graphql_sample")
            .username("graphql_sample")
            .password("himitsu"),
    )
    .await?;
    let schema = std::sync::Arc::new(Schema::new(
        Query,
        juniper::EmptyMutation::new(),
        juniper::EmptySubscription::new(),
    ));
    let context = std::sync::Arc::new(Context { pool });

    let app = axum::Router::new()
        .route(
            "/graphiql",
            axum::routing::get(|| juniper_hyper::graphiql("/graphql", None)),
        )
        .route(
            "/graphql",
            axum::routing::post(move |req| {
                juniper_hyper::graphql(schema.clone(), context.clone(), req)
            }),
        )
        .layer(tower::ServiceBuilder::new().layer(tower_http::trace::TraceLayer::new_for_http()));

    let server = if let Some(l) = listenfd::ListenFd::from_env().take_tcp_listener(0)? {
        axum::Server::from_tcp(l)?
    } else {
        axum::Server::bind(&"127.0.0.1:3000".parse()?)
    };
    server.serve(app.into_make_service()).await?;
    Ok(())
}
