#!/bin/sh

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo >&2 "Usage: $0 VERSION"
    exit 1
fi

PROJ_ROOT="$(dirname "$0")/.."
VERSION_FILE="$PROJ_ROOT"/.VERSION
OLD_VERSION=$(cat "$VERSION_FILE")

if [ "$VERSION" = "$OLD_VERSION" ]; then
    echo >&2 "New version same as old: $VERSION"
    exit 2
fi

if git status --porcelain | grep .; then
    echo >&2 Git status not clean.
    exit 3
fi

rm -f "$VERSION_FILE"
echo "$VERSION" > "$VERSION_FILE"
chmod a-w "$VERSION_FILE"
git add "$VERSION_FILE"

"$PROJ_ROOT"/bin/change_version.sh "$VERSION"

git add Cargo.toml src/lib.rs README.md
git ci -m "Version: $VERSION"
git tag v$VERSION
git push
git push --tags
