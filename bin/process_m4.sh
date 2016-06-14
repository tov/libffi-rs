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

find m4 -type f | sed 's@m4/@@' | while read file; do
    sed "s/@VERSION@/$VERSION/" "m4/$file" > "$file"
    git add "$file"
done

# git ci -m "Version: $VERSION"
# git tag v$VERSION
