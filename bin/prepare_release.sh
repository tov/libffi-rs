#!/bin/sh

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo >&2 "Usage: $0 VERSION"
    exit 1
fi

if git status --porcelain | grep .; then
    echo >&2 Git status not clean.
    exit 2
fi

find process -type f | sed 's@process/@@' | while read file; do
    sed "s/@VERSION@/$VERSION/" "process/$file" > "$file"
    git add "$file"
done

git ci -m "Version: $VERSION"
git tag v$VERSION
git push --tags
