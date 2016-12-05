# `install` phase: install stuff needed for the `script` phase

set -ex

. $(dirname $0)/utils.sh

install_c_toolchain() {
    case $TARGET in
        aarch64-unknown-linux-gnu)
            sudo apt-get install -y --no-install-recommends \
                 gcc-aarch64-linux-gnu libc6-arm64-cross libc6-dev-arm64-cross
            ;;
        *)
            # For other targets, this is handled by addons.apt.packages in .travis.yml
            ;;
    esac
}

install_rustup() {
    # uninstall the rust toolchain installed by travis, we are going to use rustup
    sh ~/rust/lib/rustlib/uninstall.sh

    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION

    rustc -V
    cargo -V
}

install_standard_crates() {
    if [ $(host) != "$TARGET" ]; then
        rustup target add $TARGET
    fi
}

configure_cargo() {
    local prefix=$(gcc_prefix)

    if [ ! -z $prefix ]; then
        # information about the cross compiler
        ${prefix}gcc -v

        # tell cargo which linker to use for cross compilation
        mkdir -p .cargo
        cat >>.cargo/config <<EOF
[target.$TARGET]
linker = "${prefix}gcc"
EOF
    fi
}

configure_postgis() {
    if [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
        # http://stackoverflow.com/questions/36875239/travis-os-x-test-postgres/36945462#36945462
        export PG_DATA=$(brew --prefix)/var/postgres
        pg_ctl -w start -l postgres.log --pgdata ${PG_DATA} || cat postgres.log
        #FATAL:  database files are incompatible with server
        #DETAIL:  The data directory was initialized by PostgreSQL version 9.4, which is not compatible with this version 9.5.4.
        #createuser -s postgres
        #cat postgres.log
    fi
    if [[ "$TRAVIS_OS_NAME" == "linux" ]]; then
        export PGUSER=postgres
        #cd src/test
        #make
        wget http://pkg.sourcepole.ch/ne_t_rex_test.dump
        pg_restore --create --no-owner -d postgres ne_t_rex_test.dump
    fi
}

main() {
    install_c_toolchain
    install_rustup
    install_standard_crates
    configure_cargo
    configure_postgis
}

main
