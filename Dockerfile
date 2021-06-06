FROM rust:1.52.1 as build-env
WORKDIR /app
ADD . /app
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/oauth2-server-rs /
EXPOSE 8080
CMD ["./oauth2-server-rs"]
