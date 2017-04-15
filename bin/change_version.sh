#!/bin/sh

VERSION=$1
if [ -z "$VERSION" ]; then
    echo >&2 "Usage: $0 VERSION"
    exit 1
fi

cd "$(dirname "$0")/.."

sed -e '/^version = "[0-9.]*"$/s/".*"/"'$VERSION'"/' \
    -i '' Cargo.toml

sed -e '/libffi = "[0-9.]*"$/s/".*"/"'$VERSION'"/' \
    -i '' src/lib.rs README.md
