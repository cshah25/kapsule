# Kapsule Development Makefile

.PHONY: build dev clean xhost help

# Default target
help:
	@echo "Kapsule Development Commands:"
	@echo "  make build    - Build the development container"
	@echo "  make dev      - Start an interactive shell inside the dev container"
	@echo "  make xhost    - Allow the container to access your X11 display"
	@echo "  make clean    - Remove the development container and volumes"

# Allow local connections to X11 (optional helper)
xhost:
	@command -v xhost >/dev/null 2>&1 && xhost +local:docker || echo "xhost not found, relying on .Xauthority"

# Build the development environment
build:
	@cd develop && docker-compose build

# Launch the development shell
dev:
	@cd develop && docker-compose run --rm kapsule-dev

# Cleanup
clean:
	@cd develop && docker-compose down -v
