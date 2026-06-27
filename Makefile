SHELL := /bin/bash
.PHONY: help test test-wiremock test-live-smoke test-live-full test-live-massive seed seed-reset lint format check ci coverage clean

help:
	@echo "lazygitlab test ground targets:"
	@echo "  make test              - wiremock + unit (fast, hermetic)"
	@echo "  make test-live-smoke   - quick live tests against test ground (--ignored live_smoke)"
	@echo "  make test-live-full    - exhaustive live tests (--ignored live_full)"
	@echo "  make test-live-massive - massive-MR diff test (requires --with-massive-mr seed)"
	@echo "  make seed              - populate test ground (idempotent)"
	@echo "  make seed-massive      - populate test ground + seed a 120-file MR"
	@echo "  make seed-reset        - DELETE test ground project (DESTRUCTIVE)"
	@echo "  make lint              - cargo clippy --all-targets -- -D warnings"
	@echo "  make format            - cargo fmt --all"
	@echo "  make check             - fmt --check + lint + test (pre-merge gate)"
	@echo "  make coverage          - cargo tarpaulin (Linux only)"

test: test-wiremock

test-wiremock:
	cargo test

test-live-smoke:
	cargo test -- --ignored live_smoke

test-live-full:
	cargo test -- --ignored live_full

test-live-massive: seed-massive
	cargo test --test test_massive_mr_diffs -- --ignored

seed:
	python scripts/populate_test_ground.py

seed-massive:
	python scripts/populate_test_ground.py --with-massive-mr

seed-reset:
	python scripts/populate_test_ground.py --reset

lint:
	cargo clippy --all-targets -- -D warnings

format:
	cargo fmt --all

check: format lint test

ci: check test-live-smoke

coverage:
	cargo tarpaulin --out Html --output-dir target/coverage --timeout 180

clean:
	cargo clean
	rm -rf data/test_ground.json target/coverage
