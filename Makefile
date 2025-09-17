run:
	docker run \
		--network=host \
		--env-file=.env \
		-it omajinai:latest

run-bg:
	docker run \
		--network=host \
		--env-file=.env \
		-d omajinai:latest

build:
	docker build -t omajinai:latest .

fmt:
	cargo +nightly fmt --all -- --emit=files
