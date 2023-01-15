# Rhizome

Slightly over-engineered personal knowledge base. Uses code and tricks from Luca Palmieri's excellent book [Zero to Production in Rust](https://www.zero2prod.com). [WIP]

## Installation

Prerequisites: `docker`, `docker-compose-plugin`.

Clone the repo, create and run the containers.

```sh
git clone https://github.com/kldtz/rhizome.git
cd rhizome && docker compose up -d
```

## Manual database backup

Use docker to copy the dump file. Going via stdout can corrupt the file.

```
DB=$(docker-compose ps -q db)

# Dump db
docker exec -i $DB db pg_dump -U postgres -F t knowledge -f knowledge_dump.tar
docker cp $DB:knowledge_dump.tar .

# Restore db
docker cp knowledge_dump.tar $DB:/
docker exec -i $DB pg_restore -U postgres -c -d knowledge knowledge_dump.tar
```

## Open tasks

* Clean up config and deal with secrets
* Graph view
* HTMX for more interactivity