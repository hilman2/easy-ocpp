FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 ocpp && useradd --uid 10001 --gid ocpp ocpp \
    && mkdir -p /app/data && chown -R ocpp:ocpp /app
COPY --from=build /src/target/release/easy-ocpp /usr/local/bin/easy-ocpp
WORKDIR /app
USER ocpp
VOLUME ["/app/data"]
EXPOSE 8080
ENTRYPOINT ["easy-ocpp"]
