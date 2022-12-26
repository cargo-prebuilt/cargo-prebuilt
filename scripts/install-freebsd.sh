#!/usr/bin/env bash

FREEBSD_MIRROR=\"$(fetch_best_freebsd_mirror)\" install_freebsd_package memstat
