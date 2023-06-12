.PHONY: build open deploy
build:
	cargo d --no-deps --workspace --target-dir ./target
	echo "<meta http-equiv=\"refresh\" content=\"0; url=substrate\">" > ./target/doc/index.html
open: build
	open ./target/doc/index.html
deploy: build
	fly deploy

