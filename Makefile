COMPOSE ?= docker compose --env-file .env -f docker/compose.yml
COMPOSE_EMBEDDING ?= docker compose --env-file .env -f docker/compose.embedding.yml
SERVICE ?= fukuyoka_app

# Dynamic user info for container
CURRENT_UID := $(shell id -u)
CURRENT_GID := $(shell id -g)
CURRENT_USER := $(shell whoami)
CURRENT_GROUP := $(shell id -gn)
BUILD_ARGS := --build-arg UID=$(CURRENT_UID) --build-arg GID=$(CURRENT_GID) --build-arg USER=$(CURRENT_USER) --build-arg GROUP=$(CURRENT_GROUP)

.PHONY: build up down logs bash parquet api test clean embedding

help:
	@echo "Makefile commands:"
	@echo "  build      - Build the Docker images"
	@echo "  up         - Start the Docker containers"
	@echo "  down       - Stop the Docker containers"
	@echo "  restart-*  - Restart a container (e.g., make restart-fukuyoka_app)"
	@echo "  in         - Access the app container shell"
	@echo "  logs       - View logs of the app container"
	@echo "  ps         - List running containers"
	@echo "  hugo       - Build the frontend with Hugo"
	@echo "  clean      - Remove containers, networks, and volumes"

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
	docker exec  fukuyoka_proxy nginx -s reload

in:
	$(COMPOSE) exec $(SERVICE) bash

ps:
	$(COMPOSE) ps
	
logs:
	$(COMPOSE) logs -f	

hugo:
	cd frontend && hugo 

clean:
	$(COMPOSE) down -v --remove-orphans

