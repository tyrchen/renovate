build:
	@cargo build
	@rm -f ~/.cargo/bin/renovate && cp ~/.target/debug/renovate ~/.cargo/bin/

test:
	@cargo nextest run --all-features

snapshot:
	@TRYCMD=overwrite cargo test --test cli_tests --all-features

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -m "Update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

.PHONY: build cov test release
