run:
	docker run \
		--network=host \
		--env-file=.env \
		-v refxpy_data:/srv/root/.data \
		-it omajinai:latest

run-bg:
	docker run \
		--network=host \
		--env-file=.env \
		-v refxpy_data:/srv/root/.data \
		-d omajinai:latest

build:
	docker build -t omajinai:latest .

fmt:
	cargo +nightly fmt --all -- --emit=files
