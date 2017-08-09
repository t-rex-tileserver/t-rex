# This script takes care of testing your crate

set -ex

main() {
    if [ ! -z $DISABLE_TESTS ]; then
        cargo build --target $TARGET
        cargo build --target $TARGET --release
        return
    fi

    #cargo test --target $TARGET
    # cross failes with linking -lgdal
    cargo test --all --target $TARGET --release

    cargo test --all --target $TARGET
    # libgdal-dev from ubuntugis drops postgresql-9.4-postgis-2.3
    if [ $TRAVIS_OS_NAME = osx ]; then
        # cross ignores DBCONN env variable (https://github.com/japaric/cross/issues/76)
        cargo test --all --target $TARGET -- --ignored
    fi

    #cargo run --target $TARGET
    cargo run --target $TARGET --release

    ldd target/$TARGET/release/t_rex
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
