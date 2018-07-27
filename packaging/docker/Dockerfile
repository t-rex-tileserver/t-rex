# t-rex container
#
# docker build -t t-rex -f Dockerfile .
# docker run t-rex --version
# docker run -p 6767:6767 -v $PWD:/var/data/in:ro -v /tmp:/var/data/out t-rex serve --bind=0.0.0.0 --openbrowser=false --datasource=data/natural_earth.gpkg

FROM ubuntu:xenial

# GDAL 2 from ubuntugis
RUN echo "deb http://ppa.launchpad.net/ubuntugis/ubuntugis-unstable/ubuntu xenial main" >> /etc/apt/sources.list
RUN gpg --keyserver keyserver.ubuntu.com --recv 314DF160
RUN gpg --export --armor 314DF160 | apt-key add -

RUN apt-get update && apt-get install -y openssl libgdal20 gdal-bin

ADD t-rex-*.deb .
RUN dpkg -i t-rex-*.deb
RUN rm t-rex-*.deb

USER www-data

WORKDIR /var/data/in

VOLUME ["/var/data/in"]
VOLUME ["/var/data/out"]

EXPOSE 6767
ENTRYPOINT ["/usr/bin/t_rex"]
