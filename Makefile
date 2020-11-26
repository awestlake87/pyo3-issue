
all:
	make .make/pyo3-issue-dev

.make/cache:
	mkdir -p .make
	touch .make/cache

.make/dev-home:
	mkdir -p .dev-home
	touch .make/dev-home

.make/dev-cargo-cache:
	mkdir -p .dev-cargo-cache
	touch .make/dev-cargo-cache

.make/dev-cargo-target:
	mkdir -p .dev-cargo-target
	touch .make/dev-cargo-target

.make/pyo3-issue-dev: \
	.make/cache \
	.make/dev-home \
	.make/dev-cargo-cache \
	.make/dev-cargo-target \
	docker/pyo3-issue-dev/* \

	docker build -f docker/pyo3-issue-dev/Dockerfile -t pyo3-issue-dev .
	touch .make/pyo3-issue-dev

clean:
	rm -rf .dev-cargo-cache
	rm -rf .dev-cargo-target
	rm -rf .dev-home
	rm -rf .make