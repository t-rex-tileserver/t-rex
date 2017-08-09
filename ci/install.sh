set -ex

configure_postgis() {
    if [ $TRAVIS_OS_NAME = osx ]; then
        # http://stackoverflow.com/questions/36875239/travis-os-x-test-postgres/36945462#36945462
        export PG_DATA=$(brew --prefix)/var/postgres
        #FATAL:  database files are incompatible with server
        #DETAIL:  The data directory was initialized by PostgreSQL version 9.4, which is not compatible with this version 9.5.4.
        rm -rf $PG_DATA
        initdb $PG_DATA -E utf8
        pg_ctl -w start -l postgres.log --pgdata ${PG_DATA}
        createuser -s postgres
    fi
    export PGUSER=postgres
    #cd src/test
    #make
    wget http://pkg.sourcepole.ch/ne_t_rex_test.dump
    pg_restore --create --no-owner -d postgres ne_t_rex_test.dump
}

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    configure_postgis

    if [ $TRAVIS_OS_NAME = linux ]; then
        sudo add-apt-repository ppa:ubuntugis/ppa --yes
        sudo apt-get --yes --force-yes update -qq
        sudo apt-get install --yes --no-install-recommends libgdal-dev
    fi
}

main
