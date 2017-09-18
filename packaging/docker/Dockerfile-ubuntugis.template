# t-rex build container
#
# docker build --build-arg uid=$UID -t t-rex-%%UBUNTU-SUITE%% -f Dockerfile-%%UBUNTU-SUITE%% .
# docker run -t -i -v $(pwd):/var/data/in t-rex-%%UBUNTU-SUITE%%
# 
# Copy generated DEB package
# docker run --entrypoint="" t-rex-%%UBUNTU-SUITE%% ls -l /
# docker run --entrypoint="" -v $(pwd):/var/data/out t-rex-%%UBUNTU-SUITE%% cp /t-rex-0.8.0-dev-amd64-ubuntugis~%%UBUNTU-SUITE%%.deb /var/data/out/

FROM ubuntu:%%UBUNTU-SUITE%%

ARG gitref
ARG uid

RUN apt-get -y update && apt-get install -y wget make

# From https://github.com/rust-lang-nursery/docker-rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    \
# this "case" statement is generated via "update.sh"
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='f5833a64fd549971be80fa42cffc6c5e7f51c4f443cd46e90e4c17919c24481f' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='67a98a67f7f7bf19c5cde166499acb8299f2f8fa88c155093df53b66da1f512a' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='82fe368c4ebf1683d57e137242793a4417042639aace8bd514601db7d79d3645' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='7a1c085591f6c1305877919f8495c04a1c97546d001d1357a7a879cedea5afbb' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    \
    url="https://static.rust-lang.org/rustup/archive/1.6.0/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain 1.20.0; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
rustc --version;

# GDAL 2 from ubuntugis
RUN echo "deb http://ppa.launchpad.net/ubuntugis/ubuntugis-unstable/ubuntu %%UBUNTU-SUITE%% main" >> /etc/apt/sources.list
RUN gpg --keyserver keyserver.ubuntu.com --recv 314DF160
RUN gpg --export --armor 314DF160 | apt-key add -

RUN apt-get update && apt-get install -y libgdal-dev

# Build t-rex
RUN wget -qO- https://api.github.com/repos/t-rex-tileserver/t-rex/tarball/${gitref:-master} | tar xzf -
RUN mv t-rex-tileserver* t-rex
RUN cd t-rex && cargo build --release

# Build DEB package
RUN apt-get update && apt-get install -y fakeroot

RUN VERSION=$(grep version t-rex/Cargo.toml | sed 's/version = "\(.*\)"/\1/') && \
    ARCH=$(dpkg --print-architecture) && \
    mkdir -p debian/usr/bin && \
    \
    install -m0755 t-rex/target/release/t_rex debian/usr/bin/ && \
    strip -s debian/usr/bin/t_rex && \
    \
    mkdir -p debian/DEBIAN && \
    echo "Package: t-rex\n\
Version: $VERSION\n\
Architecture: $ARCH\n\
Maintainer: Pirmin Kalberer <pi_deb@sourcepole.ch>\n\
Description: t-rex vector tile server\n\
Depends: openssl, libgdal20\n"\
>debian/DEBIAN/control && \
    fakeroot dpkg-deb --build debian && \
    mv debian.deb t-rex-$VERSION-$ARCH-ubuntugis~%%UBUNTU-SUITE%%.deb && \
    echo t-rex-$VERSION-$ARCH-ubuntugis-%%UBUNTU-SUITE%%.deb

RUN dpkg -i t-rex-*-ubuntugis~%%UBUNTU-SUITE%%.deb && rm -rf debian
RUN rm -rf t-rex-tileserver*/target

RUN useradd --uid ${uid:-1000} rust
USER rust

WORKDIR /var/data/in

VOLUME ["/var/data/in"]
VOLUME ["/var/data/out"]

EXPOSE 6767
ENTRYPOINT ["/usr/bin/t_rex"]
