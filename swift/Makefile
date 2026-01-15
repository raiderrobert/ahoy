# Ahoy Native macOS Notification Helper
# Build pipeline for Swift-based notification system

SWIFT_SRC = ahoy-notify.swift
SWIFT_BIN = ahoy-notify
INFO_PLIST = Info.plist
APP_BUNDLE = Ahoy.app
APP_BUNDLE_PATH = ../$(APP_BUNDLE)

# Installation paths
INSTALL_DIR = $(HOME)/.ahoy
INSTALL_APP = $(INSTALL_DIR)/$(APP_BUNDLE)

.PHONY: all clean build sign install uninstall test

all: build sign

# Compile Swift binary with embedded Info.plist
build:
	@echo "Building $(SWIFT_BIN)..."
	swiftc -O -o $(SWIFT_BIN) $(SWIFT_SRC) \
		-Xlinker -sectcreate -Xlinker __TEXT -Xlinker __info_plist -Xlinker $(INFO_PLIST)
	@echo "Copying to app bundle..."
	mkdir -p $(APP_BUNDLE_PATH)/Contents/MacOS
	mkdir -p $(APP_BUNDLE_PATH)/Contents/Resources
	cp $(SWIFT_BIN) $(APP_BUNDLE_PATH)/Contents/MacOS/
	cp -n ../resources/icons/ahoy-icon-512.png $(APP_BUNDLE_PATH)/Contents/Resources/ 2>/dev/null || true
	cp -n ../resources/icons/ahoy-icon-128.png $(APP_BUNDLE_PATH)/Contents/Resources/ 2>/dev/null || true
	@echo "Build complete."

# Code sign the app bundle
sign:
	@echo "Signing app bundle..."
	codesign --force --deep --sign - $(APP_BUNDLE_PATH)
	@echo "Registering with Launch Services..."
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f $(APP_BUNDLE_PATH)
	@echo "Signing complete."

# Install to ~/.ahoy/
install: all
	@echo "Installing to $(INSTALL_DIR)..."
	mkdir -p $(INSTALL_DIR)
	rm -rf $(INSTALL_APP)
	cp -R $(APP_BUNDLE_PATH) $(INSTALL_APP)
	/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f $(INSTALL_APP)
	@echo "Installed to $(INSTALL_APP)"

# Remove installation
uninstall:
	@echo "Removing $(INSTALL_APP)..."
	rm -rf $(INSTALL_APP)
	@echo "Uninstalled."

# Test notification
test:
	@echo "Sending test notification..."
	$(APP_BUNDLE_PATH)/Contents/MacOS/$(SWIFT_BIN) "Ahoy Test" "Makefile test successful"

# Clean build artifacts
clean:
	rm -f $(SWIFT_BIN)
	rm -rf $(APP_BUNDLE_PATH)/Contents/MacOS/$(SWIFT_BIN)
	@echo "Cleaned."
