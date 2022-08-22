FROM rust:1.63.0-slim-buster AS builder
ADD . .
RUN cargo +$(cat rust-toolchain) test && cargo +$(cat rust-toolchain) build --release
RUN ls -lha target/release
EXPOSE 8080

FROM ubuntu:22.10 AS runtime
EXPOSE 8080
WORKDIR /app
COPY --from=builder target/release/flights-rs .
RUN ls -lha
ENTRYPOINT ["./flights-rs"]
