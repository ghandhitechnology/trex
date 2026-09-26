#!/usr/bin/env bash
# Build the trailer from scratch: footage, audio, the 1080p60 render, and a
# 4K master upscaled with whole pixels (1080p is already an exact 6x).
set -euo pipefail
cd "$(dirname "$0")/.."
tools/capture.sh
tools/audio.sh --force
npx --yes hyperframes@0.8.78 render -o renders/trex-trailer-1080p60.mp4 --quality delivery
ffmpeg -loglevel error -y -i renders/trex-trailer-1080p60.mp4 -vf scale=3840:2160:flags=neighbor \
  -c:v libx264 -crf 14 -preset slow -pix_fmt yuv420p -c:a copy renders/trex-trailer-4k60.mp4
echo "renders/trex-trailer-1080p60.mp4 renders/trex-trailer-4k60.mp4"
