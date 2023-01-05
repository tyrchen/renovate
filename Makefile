build:
	@cargo build

test:
	@CELLA_ENV=test cargo nextest run --all-features

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -m "Update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

.PHONY: build cov test release
