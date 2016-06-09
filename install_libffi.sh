#!/bin/sh
set -ex
wget ftp://sourceware.org/pub/libffi/libffi-3.2.1.tar.gz
tar -zxf libffi-3.2.1.tar.gz
cd libffi-3.2.1 && ./configure --prefix=/usr && make && sudo make install
