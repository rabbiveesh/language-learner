#!/bin/sh
# Restore the database from the backup if the volume is empty, then run the server with
# Litestream replicating every write. Without a bucket, run without backups (and say so).
set -e

if [ -z "$BUCKET_NAME" ]; then
    echo "warning: no BUCKET_NAME, running WITHOUT backups" >&2
    exec fala
fi

litestream restore -if-db-not-exists -if-replica-exists /data/fala.db
exec litestream replicate -exec fala
