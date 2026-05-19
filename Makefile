run:
	docker run \
		--network=host \
		--env-file=.env \
		-v meat-my-beat-i_data:/srv/root/.data \
		-it omajinai:latest

run-bg:
	docker run \
		--network=host \
		--env-file=.env \
		-v meat-my-beat-i_data:/srv/root/.data \
		-d omajinai:latest

build:
	DOCKER_BUILDKIT=1 docker build -t omajinai:latest .

fmt:
	cargo +nightly fmt --all -- --emit=files
