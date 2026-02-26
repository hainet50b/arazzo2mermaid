FROM rust:1 AS build
COPY ./ /usr/src/arazzo2mermaid
WORKDIR /usr/src/arazzo2mermaid
RUN cargo build --release

FROM debian:trixie-slim AS runtime
COPY --from=build /usr/src/arazzo2mermaid/target/release/arazzo2mermaid /usr/local/bin/arazzo2mermaid
WORKDIR /spec
ENTRYPOINT [ "arazzo2mermaid" ]
