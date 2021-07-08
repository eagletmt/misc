use warp::Filter as _;

#[derive(Debug, juniper::GraphQLObject, sqlx::FromRow)]
#[graphql()]
struct User {
    id: i32,
    name: String,
}

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
    env_logger::init();

    let pool = sqlx::PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("graphql_sample")
            .username("postgres")
            .password("himitsu"),
    )
    .await?;
    let schema = Schema::new(
        Query,
        juniper::EmptyMutation::new(),
        juniper::EmptySubscription::new(),
    );

    let state = warp::any().map(move || Context { pool: pool.clone() });
    let log = warp::log("graphql_sample");
    let graphql_filter = juniper_warp::make_graphql_filter(schema, state.boxed());
    let graphiql = warp::get()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql", None));
    let graphql = warp::path("graphql").and(graphql_filter);
    let service = warp::service(graphiql.or(graphql).with(log));

    let server = if let Some(l) = listenfd::ListenFd::from_env().take_tcp_listener(0)? {
        hyper::Server::from_tcp(l)?
    } else {
        hyper::Server::bind(&"127.0.0.1:3000".parse()?)
    };
    server
        .serve(hyper::service::make_service_fn(|_| {
            let service = service.clone();
            async move { Ok::<_, std::convert::Infallible>(service) }
        }))
        .await?;
    Ok(())
}
