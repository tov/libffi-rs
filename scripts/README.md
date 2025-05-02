# qemutests.sh

This script uses QEMU to run tests on the following CPU architectures:

* ARMv7
* PowerPC64
* PowerPC64 little-endian
* Riscv64
* S390X
* Sparc64

The script has been tested on Debian and Ubuntu, and requires `gcc` and `rustup`
to be installed.

## Usage

Run `scripts/qemutests.sh SETUP` to set up everything needed to run tests and
run the tests.

If everything is already set up, this script can be executed with
`scripts/qemutests.sh`.