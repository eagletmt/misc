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

type ScalarValue = juniper::DefaultScalarValue;
type Schema = juniper::RootNode<
    'static,
    Query,
    juniper::EmptyMutation<Context>,
    juniper::EmptySubscription<Context>,
    ScalarValue,
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
            axum::routing::get(juniper_axum::graphiql("/graphql", None)),
        )
        .route("/graphql", axum::routing::post(graphql_handler))
        .layer(tower::ServiceBuilder::new().layer(tower_http::trace::TraceLayer::new_for_http()))
        .layer(axum::Extension(context))
        .layer(axum::Extension(schema));

    let listener = if let Some(l) = listenfd::ListenFd::from_env().take_tcp_listener(0)? {
        tokio::net::TcpListener::from_std(l)?
    } else {
        tokio::net::TcpListener::bind("127.0.0.1:3000").await?
    };
    axum::serve(listener, app).await?;
    Ok(())
}

async fn graphql_handler(
    schema: axum::Extension<std::sync::Arc<Schema>>,
    context: axum::Extension<std::sync::Arc<Context>>,
    req: juniper_axum::extract::JuniperRequest<ScalarValue>,
) -> juniper_axum::response::JuniperResponse<ScalarValue> {
    juniper_axum::response::JuniperResponse(req.0.execute(&schema, &context).await)
}
