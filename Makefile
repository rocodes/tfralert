
BINARY_NAME = tfralert
DIST_DIR = dist
EXTRA_FILES = README.md LICENSE.md

# Targets
LINUX_TARGET = x86_64-unknown-linux-gnu
MACOS_TARGET = x86_64-apple-darwin
MACOS_ARM_TARGET = aarch64-apple-darwin
WINDOWS_TARGET = x86_64-pc-windows-gnu
WASM_TARGET = wasm32-unknown-unknown

# macOS signing and notarization (TODO)
MACOS_IDENTITY ?=
APPLE_ID ?=
APPLE_TEAM_ID ?=
NOTARY_KEYCHAIN_PROFILE ?=
NOTARY_TIMEOUT ?= 300

build:
	cargo build

run:
	cargo run

release:
	cargo build --release

targets:
	rustup target add $(LINUX_TARGET)
	rustup target add $(MACOS_TARGET)
	rustup target add $(MACOS_ARM_TARGET)
	rustup target add $(WINDOWS_TARGET)
	rustup target add $(WASM_TARGET)

build-linux:
	cargo build --release --target $(LINUX_TARGET)

build-macos:
	cargo build --release --target $(MACOS_TARGET)

build-macos-arm:
	cargo build --release --target $(MACOS_ARM_TARGET)

build-windows:
	cargo build --release --target $(WINDOWS_TARGET)

# use dioxus instead of cargo build for wasm
build-wasm:
	dioxus build --platform web --release

build-all: targets build-linux build-macos build-macos-arm build-windows build-wasm

# macOS Signing (WIP)

sign-macos:
	@if [ -z "$(MACOS_IDENTITY)" ]; then \
		echo "Error: MACOS_IDENTITY not set. Example:"; \
		echo "  export MACOS_IDENTITY=\"Developer ID Application: Your Name (TEAMID)\""; \
		exit 1; \
	fi
	codesign --deep --force --options runtime --sign "$(MACOS_IDENTITY)" target/$(MACOS_TARGET)/release/$(BINARY_NAME)
	codesign --verify --verbose target/$(MACOS_TARGET)/release/$(BINARY_NAME)
	@echo "Signed macOS Intel binary."

sign-macos-arm:
	@if [ -z "$(MACOS_IDENTITY)" ]; then \
		echo "Error: MACOS_IDENTITY not set."; \
		exit 1; \
	fi
	codesign --deep --force --options runtime --sign "$(MACOS_IDENTITY)" target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME)
	codesign --verify --verbose target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME)
	@echo "Signed macOS ARM binary."

sign-all: sign-macos sign-macos-arm

# macOS Notarization (WIP)

notarize-macos:
	@if [ -z "$(NOTARY_KEYCHAIN_PROFILE)" ]; then \
		echo "Error: NOTARY_KEYCHAIN_PROFILE not set. Example:"; \
		echo "  xcrun notarytool store-credentials 'MyProfile' --apple-id 'user@apple.com' --team-id 'TEAMID' --password 'app-specific-password'"; \
		echo "  export NOTARY_KEYCHAIN_PROFILE=MyProfile"; \
		exit 1; \
	fi
	xcrun notarytool submit target/$(MACOS_TARGET)/release/$(BINARY_NAME) \
		--keychain-profile "$(NOTARY_KEYCHAIN_PROFILE)" \
		--wait --timeout $(NOTARY_TIMEOUT)
	xcrun stapler staple target/$(MACOS_TARGET)/release/$(BINARY_NAME)
	@echo "Notarized macOS Intel binary."

notarize-macos-arm:
	@if [ -z "$(NOTARY_KEYCHAIN_PROFILE)" ]; then \
		echo "Error: NOTARY_KEYCHAIN_PROFILE not set."; \
		exit 1; \
	fi
	xcrun notarytool submit target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME) \
		--keychain-profile "$(NOTARY_KEYCHAIN_PROFILE)" \
		--wait --timeout $(NOTARY_TIMEOUT)
	xcrun stapler staple target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME)
	@echo "Notarized macOS ARM binary."

notarize-all: notarize-macos notarize-macos-arm

# Packaging

package: clean-dist build-all ## sign-all notarize-all (TODO)
	mkdir -p $(DIST_DIR)

	cp target/$(LINUX_TARGET)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux
	cp $(EXTRA_FILES) $(DIST_DIR)/
	tar -czf $(DIST_DIR)/$(BINARY_NAME)-linux.tar.gz -C $(DIST_DIR) $(BINARY_NAME)-linux $(notdir $(EXTRA_FILES))
	rm $(DIST_DIR)/$(BINARY_NAME)-linux

	cp target/$(MACOS_TARGET)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-macos
	tar -czf $(DIST_DIR)/$(BINARY_NAME)-macos.tar.gz -C $(DIST_DIR) $(BINARY_NAME)-macos $(notdir $(EXTRA_FILES))
	rm $(DIST_DIR)/$(BINARY_NAME)-macos

	cp target/$(MACOS_ARM_TARGET)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-macos-arm
	tar -czf $(DIST_DIR)/$(BINARY_NAME)-macos-arm.tar.gz -C $(DIST_DIR) $(BINARY_NAME)-macos-arm $(notdir $(EXTRA_FILES))
	rm $(DIST_DIR)/$(BINARY_NAME)-macos-arm

	cp target/$(WINDOWS_TARGET)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-windows.exe
	zip -j $(DIST_DIR)/$(BINARY_NAME)-windows.zip $(DIST_DIR)/$(BINARY_NAME)-windows.exe $(EXTRA_FILES)
	rm $(DIST_DIR)/$(BINARY_NAME)-windows.exe

	rm -f $(foreach file,$(EXTRA_FILES),$(DIST_DIR)/$(notdir $(file)))


clean:
	cargo clean

clean-dist:
	rm -rf $(DIST_DIR)

help:
	@echo "Available targets:"
	@echo "  make build-all              Build for all platforms"
	@echo "  make sign-all               Sign both macOS binaries"
	@echo "  make notarize-all           Notarize macOS Intel and ARM builds"
	@echo "  make package                Build, sign, notarize and package"
	@echo "  make clean                  Clean cargo artifacts"
	@echo "  make clean-dist             Clean dist folder"
	@echo
	@echo "Environment variables:"
	@echo "  MACOS_IDENTITY              Your macOS Developer ID Application identity"
	@echo "  NOTARY_KEYCHAIN_PROFILE     Your notarization keychain profile name"
	@echo "  APPLE_ID, APPLE_TEAM_ID     Optional for notarization if using direct credentials"
