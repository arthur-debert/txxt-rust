#! /usr/bin/env bash
# This setups basic rust enviroment
# if rustup is already installed , return early
if command -v rustup &>/dev/null; then
    return 0 2>/dev/null || exit 0
fi

# if macos, use brew, if linux, use apt-get
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    sudo apt-get install -y rustup build-essential
elif [[ "$OSTYPE" == "darwin"* ]]; then
    brew install rustup
else
    echo "Unsupported OS: $OSTYPE"
    return 1 2>/dev/null || exit 1
fi

# setup a default rust enviroment
rustup default stable
# add clippy and rustfmt
rustup component add clippy rustfmt\n
# the clippy and rustfmr config files are in ${PRJECT_ROOT}
