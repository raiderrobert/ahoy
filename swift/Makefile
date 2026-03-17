# Ahoy — native macOS notification CLI
# Single Swift binary, no package manager

SWIFT_SRC = ahoy.swift
SWIFT_BIN = ahoy
INFO_PLIST = Info.plist
APP_BUNDLE = Ahoy.app
APP_BUNDLE_PATH = ../$(APP_BUNDLE)

INSTALL_DIR = $(HOME)/.ahoy
INSTALL_APP = $(INSTALL_DIR)/$(APP_BUNDLE)
INSTALL_BIN = $(INSTALL_DIR)/bin/$(SWIFT_BIN)

.PHONY: all clean build sign install uninstall test

all: build sign

build:
	@echo "Building $(SWIFT_BIN)..."
	swiftc -O -target arm64-apple-macos13.0 -o $(SWIFT_BIN) $(SWIFT_SRC) \
		-Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker $(INFO_PLIST)
	@echo "Copying to app bundle..."
	mkdir -p $(APP_BUNDLE_PATH)/Contents/MacOS
	mkdir -p $(APP_BUNDLE_PATH)/Contents/Resources
	cp $(SWIFT_BIN) $(APP_BUNDLE_PATH)/Contents/MacOS/
	cp $(INFO_PLIST) $(APP_BUNDLE_PATH)/Contents/Info.plist
	cp -n icons/ahoy-icon-512.png $(APP_BUNDLE_PATH)/Contents/Resources/ 2>/dev/null || true
	cp -n icons/ahoy-icon-128.png $(APP_BUNDLE_PATH)/Contents/Resources/ 2>/dev/null || true
	@echo "Build complete."

sign:
	@echo "Signing app bundle..."
	codesign --force --deep --sign - $(APP_BUNDLE_PATH)
	@echo "Registering with Launch Services..."
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f $(APP_BUNDLE_PATH)
	@echo "Signing complete."

install: all
	@echo "Installing to $(INSTALL_DIR)..."
	mkdir -p $(INSTALL_DIR)/bin
	rm -rf $(INSTALL_APP)
	cp -R $(APP_BUNDLE_PATH) $(INSTALL_APP)
	ln -sf $(INSTALL_APP)/Contents/MacOS/$(SWIFT_BIN) $(INSTALL_BIN)
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f $(INSTALL_APP)
	@echo "Installed to $(INSTALL_APP)"
	@echo "Symlinked $(INSTALL_BIN) → $(INSTALL_APP)/Contents/MacOS/$(SWIFT_BIN)"

uninstall:
	@echo "Removing $(INSTALL_APP)..."
	rm -rf $(INSTALL_APP)
	rm -f $(INSTALL_BIN)
	@echo "Uninstalled."

test: build
	@echo "Running integration tests..."
	bash ../tests/integration.sh $(APP_BUNDLE_PATH)/Contents/MacOS/$(SWIFT_BIN)

clean:
	rm -f $(SWIFT_BIN)
	rm -rf $(APP_BUNDLE_PATH)/Contents/MacOS/$(SWIFT_BIN)
	@echo "Cleaned."
