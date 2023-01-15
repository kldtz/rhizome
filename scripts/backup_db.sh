#!/usr/bin/env bash
set -x
set -eo pipefail

start_pwd=`pwd`
message="${1:-Backup}"

function backup {
 # Get container hash
 DB=$(docker compose ps -q db)

 # Dump db
 docker exec -i $DB pg_dump -U postgres -F t knowledge -f knowledge_dump.tar
 docker cp $DB:knowledge_dump.tar backup/
}

function cleanup {
  cd $start_pwd
}

function save {
  git add .
  git commit -m "$message"
  git push
}

backup
cd backup
save

trap cleanup EXIT

