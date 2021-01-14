FROM rust:1.49 as builder

WORKDIR /usr/src/slack-slash-command-sample
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
  echo 'fn main() {}' > src/main.rs && \
  cargo build --release --locked && \
  rm -r src
COPY src ./src/
RUN cargo build --release --frozen --locked

FROM public.ecr.aws/lambda/provided:al2
COPY --from=builder /usr/src/slack-slash-command-sample/target/release/handler /var/runtime/bootstrap
CMD ["app.handler"]
