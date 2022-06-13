FROM public.ecr.aws/docker/library/rust:1.61 as builder

ARG BUILD_TYPE=--release
WORKDIR /usr/src/slack-slash-command-sample
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
  echo 'fn main() {}' > src/main.rs && \
  cargo build ${BUILD_TYPE} --locked && \
  rm -r src
COPY src ./src/
RUN cargo build ${BUILD_TYPE} --frozen --locked

FROM public.ecr.aws/lambda/provided:al2
COPY --from=builder /usr/src/slack-slash-command-sample/target/*/handler /var/runtime/bootstrap
CMD ["app.handler"]
