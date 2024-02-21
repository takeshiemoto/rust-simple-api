build:
	docker-compose build

db:
	docker-compose up

test:
	cargo test

test-s:
	cargo test --no-default-features

dev:
	sqlx db create
	sqlx migrate run
	cargo watch -x run

clean:
	docker system prune --all --volumes