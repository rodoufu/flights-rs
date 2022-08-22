build:
	docker build -f Dockerfile -t flights-rs:latest .

clean:
	cargo clean
	docker container rm flights-rs || true
	docker image rm flights-rs:latest || true

run:
	docker run --rm --name "flights-rs" -p 8080:8080 flights-rs:latest


test:
	cargo +$(cat rust-toolchain) test
