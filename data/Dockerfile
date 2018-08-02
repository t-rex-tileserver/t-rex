# PostgreSQL/PostGIS database with test data

FROM postgres:9.6

# https://hub.docker.com/_/postgres/
# Debian stretch with GDAL 2.1

ARG POSTGIS_VERSION=2.3
RUN apt-get update &&\
    DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -y \
    postgresql-contrib-$PG_MAJOR=$PG_VERSION \
    postgresql-$PG_MAJOR-postgis-$POSTGIS_VERSION \
    postgresql-$PG_MAJOR-postgis-$POSTGIS_VERSION-scripts \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# allow the container to be started with `--user`
RUN chmod g=u /etc/passwd \
&& sed -i '/# allow the container to be started with `--user`/a if ! whoami &> /dev/null; then\n\tif [ -w /etc/passwd ]; then\n\t\techo "${USER_NAME:-default}:x:$(id -u):0:${USER_NAME:-default} user:${HOME}:/sbin/nologin" >> /etc/passwd\n\tfi\nfi' /usr/local/bin/docker-entrypoint.sh

#RUN localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8
#ENV LANG en_US.utf8

RUN apt-get update && apt-get install -y make gdal-bin

# setup database
COPY *.gpkg g1k18.* /
COPY Makefile /
COPY setup-db.sh /docker-entrypoint-initdb.d/
RUN chmod +x /docker-entrypoint-initdb.d/*.sh
# Load data into DB at build time
RUN head -n -1 /usr/local/bin/docker-entrypoint.sh >/tmp/docker-entrypoint.sh
ENV PGDATA /var/lib/postgresql/docker
ENV POSTGRES_PASSWORD Uenz9mrkoRnt
RUN gosu postgres bash /tmp/docker-entrypoint.sh postgres
RUN rm /Makefile /*.gpkg /g1k18.*
