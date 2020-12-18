# t-rex container
#
# docker build -t t-rex -f Dockerfile .
# docker run t-rex --version
# docker run -p 6767:6767 -v $PWD:/var/data/in:ro -v /tmp:/var/data/out t-rex serve --bind=0.0.0.0 --openbrowser=false --datasource=data/natural_earth.gpkg

FROM ubuntu:focal

RUN apt-get update && apt-get install -y libssl1.1 gdal-bin

ADD t-rex_*.deb /
RUN dpkg -i /t-rex_*.deb
RUN rm /t-rex_*.deb

USER www-data

WORKDIR /var/data/in

VOLUME ["/var/data/in"]
VOLUME ["/var/data/out"]

EXPOSE 6767
ENTRYPOINT ["/usr/bin/t_rex"]
