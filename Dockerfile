FROM rust:latest as builder
RUN apt-get update && apt-get install -y librust-clang-sys-dev
WORKDIR /build
COPY . .
RUN rustup toolchain install nightly
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked cargo-leptos
RUN cargo leptos build --release

# debian release as the same as golang image
# set TimeZone as Asia/Shanghai
# set Local as zh-hans
FROM debian:bullseye
RUN set -ex; \
	apt-get update; \
	apt-get install -y --no-install-recommends \
	    tzdata \
	    locales \
	    ca-certificates;
RUN locale-gen zh_CN.UTF-8; \
    update-locale zh_CN.UTF-8;
RUN cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime;
ENV TZ Asia/Shanghai
ENV LANG zh_US.utf8
COPY --from=builder /build/target/server/release/hj /usr/local/bin/hj
COPY --from=builder /build/target/site /webser/www
ENV LEPTOS_OUTPUT_NAME hj
ENV LEPTOS_SITE_ROOT /webser/www
ENTRYPOINT ["hj"]
