COMPOSE ?= docker compose --env-file .env -f compose.yml
COMPOSE_DEV ?= $(COMPOSE) -f compose.override.yml
SERVICE ?= fukuyoka_app

# Dynamic user info for container
CURRENT_UID := $(shell id -u)
CURRENT_GID := $(shell id -g)
CURRENT_USER := $(shell whoami)
CURRENT_GROUP := $(shell id -gn)
BUILD_ARGS := --build-arg UID=$(CURRENT_UID) --build-arg GID=$(CURRENT_GID) --build-arg USER=$(CURRENT_USER) --build-arg GROUP=$(CURRENT_GROUP)

.PHONY: build up down logs in prepare ps hugo clean help \
        restart restart-proxy restart-db \
        db-shell db-logs \
        dev dev-build dev-down dev-logs

help:
	@echo "Makefile commands:"
	@echo "  build          - Build the Docker images"
	@echo "  up             - Start the Docker containers"
	@echo "  down           - Stop the Docker containers"
	@echo "  restart        - Restart the app container"
	@echo "  restart-proxy  - Reload nginx config"
	@echo "  restart-db     - Restart the database container"
	@echo "  in             - Access the app container shell"
	@echo "  prepare        - Generate posts.json for PostgreSQL import"
	@echo "  logs           - View logs of all containers"
	@echo "  ps             - List running containers"
	@echo "  hugo           - Build the frontend with Hugo"
	@echo "  db-shell       - Open psql shell in the database container"
	@echo "  db-logs        - View database container logs"
	@echo "  clean          - Remove containers, networks, and volumes"
	@echo ""
	@echo "Development (localhost):"
	@echo "  dev-build      - Build images for local dev"
	@echo "  dev            - Start containers for local dev (http://localhost:51841)"
	@echo "  dev-down       - Stop dev containers"
	@echo "  dev-logs       - View dev container logs"

## Development (localhost)
dev-build:
	$(COMPOSE_DEV) build $(BUILD_ARGS)

dev:
	$(COMPOSE_DEV) up -d
	echo "you can access http://localhost:51841"

dev-down:
	$(COMPOSE_DEV) down

dev-logs:
	$(COMPOSE_DEV) logs -f

clean:
	$(COMPOSE) down -v --remove-orphans

build:
	$(COMPOSE) build $(BUILD_ARGS)

up:
	$(COMPOSE) up -d

down:
	$(COMPOSE) down

restart:
	$(COMPOSE) down fukuyoka_app
	$(COMPOSE) up -d fukuyoka_app

restart-proxy:
	docker exec fukuyoka_proxy nginx -s reload

restart-db:
	$(COMPOSE) restart db

in:
	$(COMPOSE) exec $(SERVICE) bash

ps:
	$(COMPOSE) ps

logs:
	$(COMPOSE) logs -f

hugo:
	$(COMPOSE) exec -it fukuyoka_frontend hugo

prepare:
	$(COMPOSE) exec $(SERVICE) bash -c "cargo build --release --bin prepare && ./target/release/prepare --all"

db-shell:
	$(COMPOSE) exec db psql -U $${POSTGRES_USER} -d $${POSTGRES_DB}

db-logs:
	$(COMPOSE) logs -f db

