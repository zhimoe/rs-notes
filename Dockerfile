# pull the latest version of Rust
FROM rust:latest AS builder

# create a new empty shell project
RUN USER=root cargo new --bin prj
WORKDIR /prj

# copy over your manifests
COPY ./Cargo.lock ./Cargo.toml ./

# change the crate.io source
COPY ./config $CARGO_HOME/

# this build step will cache your dependencies
RUN cargo build --release
RUN rm -r src/*

# copy your source files to WORKDIR/src
COPY ./src ./src
COPY ./static ./static

# build for release, note! the Cargo.toml package name in deps is _, not -
RUN rm ./target/release/deps/rs_notes*
RUN cargo build --release

RUN mv ./target/release/rs-notes .

## 2 stage build
# our final base
FROM debian:stretch-slim AS app


# for connecting to postgres and TLS hosts
# RUN apt update -y && apt install -y libpq-dev openssl libssl1.0-dev ca-certificates

# copy the build artifact and static resources from the build stage
COPY --from=builder /prj/rs-notes ./
COPY --from=builder /prj/static ./static

# set the startup command to run your binary
CMD ["./rs-notes"]
