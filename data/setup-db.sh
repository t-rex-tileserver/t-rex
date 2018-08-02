#!/bin/bash
set -e

psql -c "CREATE ROLE t_rex SUPERUSER LOGIN PASSWORD 't_rex'"

export PGUSER=t_rex
export PGDATABASE=t_rex_tests
make createdb loaddata

psql -c "ALTER ROLE t_rex NOSUPERUSER"
