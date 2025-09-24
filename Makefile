COMPOSE ?= docker compose --env-file .env -f docker/compose.yml
SERVICE ?= fukuyoka_app

.PHONY: build up down logs bash parquet api test clean

help:
	@echo "Makefile commands:"
	@echo "  build      - Build the Docker images"
	@echo "  up         - Start the Docker containers"
	@echo "  down       - Stop the Docker containers"
	@echo "  in         - Access the app container shell"
	@echo "  logs       - View logs of the app container"
	@echo "  ps         - List running containers"
	@echo "  hugo       - Build the frontend with Hugo"
	@echo "  clean      - Remove containers, networks, and volumes"

build:
	$(COMPOSE) build

up:
	$(COMPOSE) up -d 

down:
	$(COMPOSE) down

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
