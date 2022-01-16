# graphql-sample
Boilerplate code for building web app using GraphQL

## Libraries
- Server side
    - Juniper
    - Axum
    - SQLx
- Client side
    - graphql-client
    - Yew

## Prerequisites
- Install graphql-client CLI
    - `cargo install graphql_client_cli`
- Install [trunk](https://trunkrs.dev/)

## Development
- Launch GraphQL server with `cd sample-server; systemfd --no-pid -s http::3000 -- cargo watch -x run`
- Generate schema and client code with `graphql-client introspect-schema http://localhost:3000/graphql --output schema.json`
- Launch trunk server with `cd sample-client; trunk serve`
- Open http://localhost:8080 to see main page
- Open http://localhost:3000/graphiql to use GraphiQL
