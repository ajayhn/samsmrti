#!/usr/bin/env bash
# Builds the review-only PWA and syncs it to the GCS bucket that serves it.
#
# The bucket is public (allUsers: objectViewer) and accessed via its
# virtual-hosted-style URL, not the path-style one -- the built app's asset
# references are root-absolute (e.g. /assets/foo.js), so it must be served
# from an origin whose root *is* the bucket. That's what
# https://BUCKET.storage.googleapis.com/ gives you; the path-style
# https://storage.googleapis.com/BUCKET/... does not (assets 404 there).
#
# Requires: gcloud CLI authenticated (`gcloud auth login`) with access to
# the GCS bucket below, and the Homebrew LLVM toolchain for the wasm32 build
# (see DEVELOPMENT.md).
set -euo pipefail

BUCKET="samsmrti-hosting"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v gcloud >/dev/null 2>&1; then
  export PATH="/opt/homebrew/share/google-cloud-sdk/bin:$PATH"
fi
if ! command -v gcloud >/dev/null 2>&1; then
  echo "gcloud not found. Install with: brew install --cask google-cloud-sdk" >&2
  exit 1
fi

cd "$REPO_ROOT"

export CC_wasm32_unknown_unknown="/opt/homebrew/opt/llvm/bin/clang"
export AR_wasm32_unknown_unknown="/opt/homebrew/opt/llvm/bin/llvm-ar"

npm run build:web

gcloud storage rsync dist-web "gs://${BUCKET}" --recursive --delete-unmatched-destination-objects

echo ""
echo "Deployed: https://${BUCKET}.storage.googleapis.com/index.web.html"
