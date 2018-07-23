set -ex

configure_postgis() {
    if [ $TRAVIS_OS_NAME = linux ]; then
        export PGUSER=postgres
        cd t-rex-service/src/test
        # PostGIS test can't be run, because libgdal-dev from ubuntugis drops postgresql-9.4-postgis-2.3
        # make createdb loadwgdal1
    fi
}

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    ogr2ogr --version
    configure_postgis

    if [ $TRAVIS_OS_NAME = linux ]; then
        # We want GDAL 2.x
        # https://launchpad.net/~ubuntugis/+archive/ubuntu/ppa/+packages?field.status_filter=published&field.series_filter=trusty
        sudo add-apt-repository ppa:ubuntugis/ppa --yes
        sudo apt-get --yes --force-yes update -qq
        sudo apt-get install --yes --no-install-recommends libgdal-dev
    fi
}

main
