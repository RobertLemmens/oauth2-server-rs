FROM rust:1.52.1 as build-env
WORKDIR /app
ADD . /app
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/oauth2-server-rs /
CMD ["./oauth2-server-rs"]
