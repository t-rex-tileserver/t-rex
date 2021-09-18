# t-rex container

FROM ubuntu:focal

RUN apt-get update && apt-get install -y curl libssl1.1 gdal-bin

ARG VERSION_TAG

RUN curl -o t-rex.deb -L \
    https://github.com/t-rex-tileserver/t-rex/releases/download/v${VERSION_TAG}/t-rex-focal_${VERSION_TAG}-1~focal_amd64.deb && \
    dpkg -i t-rex.deb && \
    rm t-rex.deb

USER www-data

WORKDIR /var/data/in

VOLUME ["/var/data/in"]
VOLUME ["/var/data/out"]

EXPOSE 6767
ENTRYPOINT ["/usr/bin/t_rex"]
