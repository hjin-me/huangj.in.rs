FROM hjin/leptos-build:2023063010064505b9e9 as builder
WORKDIR /build
COPY . .
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
COPY --from=builder /build/target/server/release/serv /usr/local/bin/blog-serv
COPY --from=builder /build/target/site /webser/www
ENV LEPTOS_OUTPUT_NAME hj
ENV LEPTOS_SITE_ROOT /webser/www
ENV LEPTOS_SITE_ADDR 0.0.0.0:3000
EXPOSE 3000
ENTRYPOINT ["blog-serv"]
