# graphql-sample
Boilerplate code for building web app using GraphQL

## Libraries
- Server side
    - Juniper
    - Warp
    - SQLx
- Client side
    - GraphQL Code Generator
    - graphql-request
    - React

## Development
- Launch GraphQL server with `systemfd --no-pid -s http::3000 -- cargo watch -x run`
- Generate schema and client code with `yarn run graphql-codegen`
- Launch Webpack server with `yarn run webpack serve`
- Open http://localhost:8080 to see main page
- Open http://localhost:3000/graphiql to use GraphiQL
