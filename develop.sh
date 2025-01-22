#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

# Export RUST_LOG for more detailed logging
export RUST_LOG="debug,hyper=info,mio=info"

# Start trunk with more verbose output
trunk serve --verbose --proxy-backend="https://[::1]:8443/api/" &

# Start the backend with additional debugging
RUST_BACKTRACE=1 cargo watch -x "run --bin server -- --port 8081 --ssl-port 8443 --static-dir client/dist --cert-dir cert"