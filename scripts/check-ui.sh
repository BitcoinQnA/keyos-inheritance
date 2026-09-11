#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target/previews
viewer=(foundation-slint-viewer tests/previews.slint -L ui=target/foundation/ui/ui -L theme=target/foundation/themes/slint)
"${viewer[@]}" --check
for height in 760 800; do
  for dark in false true; do
    for page in {0..19}; do
      printf '{"window-height":%s,"dark":%s,"page":%s}' "$height" "$dark" "$page" |
        "${viewer[@]}" --load-data - --screenshot "target/previews/page-${page}-${height}-${dark}.png"
    done
    for setup in {1..4}; do
      printf '{"window-height":%s,"dark":%s,"page":12,"setup":%s}' "$height" "$dark" "$setup" |
        "${viewer[@]}" --load-data - --screenshot "target/previews/setup-${setup}-${height}-${dark}.png"
    done
    for rehearsal in 1 2 3 4; do
      printf '{"window-height":%s,"dark":%s,"page":16,"rehearsal":%s}' "$height" "$dark" "$rehearsal" |
        "${viewer[@]}" --load-data - --screenshot "target/previews/rehearsal-${rehearsal}-${height}-${dark}.png"
    done
    for page in 0 4 11; do
      printf '{"window-height":%s,"dark":%s,"page":%s,"long-text":true}' "$height" "$dark" "$page" |
        "${viewer[@]}" --load-data - --screenshot "target/previews/long-${page}-${height}-${dark}.png"
    done
    printf '{"window-height":%s,"dark":%s,"page":0,"menu":true}' "$height" "$dark" |
      "${viewer[@]}" --load-data - --screenshot "target/previews/menu-${height}-${dark}.png"
    printf '{"window-height":%s,"dark":%s,"page":0,"populated":false}' "$height" "$dark" |
      "${viewer[@]}" --load-data - --screenshot "target/previews/welcome-${height}-${dark}.png"
  done
done
