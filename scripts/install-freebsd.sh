#!/usr/bin/env bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
. lib.sh
# shellcheck disable=SC1091
. freebsd-common.sh
# shellcheck disable=SC1091
. freebsd-install.sh

main() {
    apt-get update && apt-get install --assume-yes --no-install-recommends \
        curl \
        dnsutils \
        jq \
        xz-utils

    local url=
    url=$(fetch_best_freebsd_mirror)
    FREEBSD_MIRROR="${url}" setup_freebsd_packagesite
    FREEBSD_MIRROR="${url}" install_freebsd_package memstat

    rm "${0}"
}

main "${@}"