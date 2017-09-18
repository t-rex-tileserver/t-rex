# t-rex container
#
# docker build --build-arg uid=$UID -t t-rex -f Dockerfile .
# docker run -v $(pwd):/var/data/in t-rex --version
# docker run -v $(pwd):/var/data/in t-rex serve --openbrowser=false --datasource=t-rex-gdal/natural_earth.gpkg

FROM ubuntu:xenial

ARG uid

# GDAL 2 from ubuntugis
RUN echo "deb http://ppa.launchpad.net/ubuntugis/ubuntugis-unstable/ubuntu xenial main" >> /etc/apt/sources.list
RUN gpg --keyserver keyserver.ubuntu.com --recv 314DF160
RUN gpg --export --armor 314DF160 | apt-key add -

RUN apt-get update && apt-get install -y openssl libgdal20 gdal-bin

ADD t-rex-*.deb .
RUN dpkg -i t-rex-*.deb
RUN rm t-rex-*.deb

RUN useradd --uid ${uid:-1000} rust
USER rust

WORKDIR /var/data/in

VOLUME ["/var/data/in"]
VOLUME ["/var/data/out"]

EXPOSE 6767
ENTRYPOINT ["/usr/bin/t_rex"]
