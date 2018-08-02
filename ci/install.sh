set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    if [ $TRAVIS_OS_NAME = linux ]; then
        # We want GDAL 2.x
        # https://launchpad.net/~ubuntugis/+archive/ubuntu/ppa/+packages?field.status_filter=published&field.series_filter=trusty
        sudo add-apt-repository ppa:ubuntugis/ppa --yes
        sudo apt-get --yes --force-yes update -qq
        sudo apt-get install --yes --no-install-recommends libgdal-dev
    fi
}

main
