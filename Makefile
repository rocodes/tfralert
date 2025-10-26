BINARY_NAME = tfr_alert

MACOS_IDENTITY ?=
APPLE_ID ?=
APPLE_TEAM_ID ?=
NOTARY_KEYCHAIN_PROFILE ?=
NOTARY_TIMEOUT ?= 300

.PHONY: help build build-linux build-windows build-macos build-macos-arm build-bot package clean

help: ## Print help message and exit
	@printf "Subcommands:\n\n"
	@perl -ne 'if (/^([a-zA-Z0-9._-]+):.*##\s*(.*)/) { print "$$1:$$2\n" }' $(MAKEFILE_LIST) \
		| sort \
		| column -s ':' -t

deps-windows:  # Install Windows build dependencies for Linux systems
	sudo apt install g++-mingw-w64-x86-64 gcc-mingw-w64-x86-64 -y

build: ## Build crates, target host platform
	cargo build --workspace --release

build-linux: ## Build tfr_app for Linux
	cargo build -p tfr_app --release --target x86_64-unknown-linux-gnu

build-windows: ## Build tfr_app for Windows
	cargo build -p tfr_app --release --target x86_64-pc-windows-gnu

build-macos: ## Build tfr_app for macOS Intel
	cargo build -p tfr_app --release --target x86_64-apple-darwin

build-macos-arm: ## Build tfr_app for macOS ARM
	cargo build -p tfr_app --release --target aarch64-apple-darwin

build-bot: ## Build tfr_bot (Linux)
	cargo build -p tfr_bot --release --target x86_64-unknown-linux-gnu

package: ## Build packages with dist (cargo-dist) (see Cargo.toml)
	dist build

clean: ## Clean all build artifacts
	cargo clean

clean-dist:
	rm -rf $(DIST_DIR)

sign-macos:
	@if [ -z "$(MACOS_IDENTITY)" ]; then echo "Error: MACOS_IDENTITY not set."; exit 1; fi
	codesign --deep --force --options runtime --sign "$(MACOS_IDENTITY)" target/$(MACOS_TARGET)/release/$(BINARY_NAME)
	codesign --verify --verbose target/$(MACOS_TARGET)/release/$(BINARY_NAME)
	@echo "Signed macOS Intel binary."

sign-macos-arm:
	@if [ -z "$(MACOS_IDENTITY)" ]; then echo "Error: MACOS_IDENTITY not set."; exit 1; fi
	codesign --deep --force --options runtime --sign "$(MACOS_IDENTITY)" target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME)
	codesign --verify --verbose target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME)
	@echo "Signed macOS ARM binary."

sign-all: sign-macos sign-macos-arm

notarize-macos:
	@if [ -z "$(NOTARY_KEYCHAIN_PROFILE)" ]; then echo "Error: NOTARY_KEYCHAIN_PROFILE not set."; exit 1; fi
	xcrun notarytool submit target/$(MACOS_TARGET)/release/$(BINARY_NAME) \
		--keychain-profile "$(NOTARY_KEYCHAIN_PROFILE)" --wait --timeout $(NOTARY_TIMEOUT)
	xcrun stapler staple target/$(MACOS_TARGET)/release/$(BINARY_NAME)
	@echo "Notarized macOS Intel binary."

notarize-macos-arm:
	@if [ -z "$(NOTARY_KEYCHAIN_PROFILE)" ]; then echo "Error: NOTARY_KEYCHAIN_PROFILE not set."; exit 1; fi
	xcrun notarytool submit target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME) \
		--keychain-profile "$(NOTARY_KEYCHAIN_PROFILE)" --wait --timeout $(NOTARY_TIMEOUT)
	xcrun stapler staple target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME)
	@echo "Notarized macOS ARM binary."

notarize-all: notarize-macos notarize-macos-arm