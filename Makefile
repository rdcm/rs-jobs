docker-build:
	DOCKER_BUILDKIT=1 docker compose build --progress=plain --no-cache

docker-up:
	docker compose up -d

docker-down:
	docker compose down

format:
	cargo fmt

up:
	drivers/chromedriver-mac-x64/chromedriver --port=9515