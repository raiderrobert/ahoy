# List available recipes
default:
    @just --list

# Run all checks (build + test)
check:
    make -C swift build sign
    make -C swift test

# Run tests
test:
    make -C swift test

# Build release binary
build:
    make -C swift build sign

# Install locally
install:
    make -C swift install
