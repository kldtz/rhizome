# Rhizome

Slightly over-engineered personal knowledge base. Uses code and tricks from Luca Palmieri's excellent book [Zero to Production in Rust](https://www.zero2prod.com).

## Installation

Prerequisites: `docker`, `docker-compose-plugin`.

Clone the repo, create and run the containers.

```sh
git clone https://github.com/kldtz/rhizome.git
cd rhizome

# Public server
docker compose up -d

# Local deployment
docker compose -f docker-compose-local.yml up -d
```

## Manual database backup

Use docker to copy the dump file. Going via stdout can corrupt the file.

```
# Backup db
./scripts/backup_db.sh

# Get container hash
DB=$(docker compose ps -q db)

# Restore db
docker cp knowledge_dump.tar $DB:/
docker exec -i $DB pg_restore -U postgres -c -d knowledge backup/knowledge_dump.tar
```

## Open tasks

* Clean up config and deal with secrets
* Graph view
* HTMX for more interactivity