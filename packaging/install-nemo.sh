#!/usr/bin/env bash
# Installs the right-click entry for Nemo (Linux Mint): same as install-nautilus.sh with CITRUS_FM=nemo.
# Usage / Uso: packaging/install-nemo.sh [path/to/citrus]
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CITRUS_FM=nemo exec bash "$here/install-nautilus.sh" "$@"
