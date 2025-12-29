COMPOSE ?= docker compose --env-file .env -f docker/compose.yml
COMPOSE_EMBEDDING ?= docker compose --env-file .env -f docker/compose.embedding.yml
SERVICE ?= fukuyoka_app

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
	@echo "  embedding  - Build and access the embedding container"

build:
	$(COMPOSE) build

up:
	$(COMPOSE) up -d 

down:
	$(COMPOSE) down

restart-%:
	$(COMPOSE) restart $*

in:
	$(COMPOSE) exec $(SERVICE) bash

ps:
	$(COMPOSE) ps
	
logs:
	$(COMPOSE) logs -f $(SERVICE)

hugo:
	cd frontend && hugo 

clean:
	$(COMPOSE) down -v --remove-orphans

embedding:
	$(COMPOSE_EMBEDDING) build
	$(COMPOSE_EMBEDDING) run --rm embedding /bin/bash

embedding-down:
	$(COMPOSE_EMBEDDING) down
