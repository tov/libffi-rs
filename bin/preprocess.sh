#!/bin/sh

PROJ_ROOT="$(dirname "$0")/.."
VERSION_FILE="$PROJ_ROOT"/.VERSION
VERSION=$(cat "$VERSION_FILE")

find process -type f | sed -E 's@process/@@;/(^|\/)\./d' |
while read file; do
    echo "Preprocessing: $file"
    rm -f "$file"
    sed "s/@VERSION@/$VERSION/" "process/$file" > "$file"
    chmod a-w "$file"
    git add "$file"
done
