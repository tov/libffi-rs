#!/bin/sh

VERSION="$1"
if [ -z "$VERSION" ]; then
    echo >&2 "Usage: $0 VERSION"
    exit 1
fi

