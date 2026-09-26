#!/usr/bin/env bash
# Rebuild the trailer audio: cues -> music (if missing or --force) -> mix.
set -euo pipefail
cd "$(dirname "$0")/.."
node tools/cues.mjs
if [[ "${1:-}" == "--force" || ! -f assets/audio/music.wav ]]; then node tools/music.mjs; fi
node tools/mix.mjs
