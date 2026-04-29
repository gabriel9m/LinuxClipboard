#!/usr/bin/env bash
set -euo pipefail

binary_path="${HOME}/.local/bin/linuxclipboard"
desktop_file="${HOME}/.config/autostart/linuxclipboard.desktop"

if [[ -x "${binary_path}" ]]; then
    "${binary_path}" --quit || true
fi

rm -f "${binary_path}" "${desktop_file}"

echo "Removed ${binary_path}"
echo "Removed ${desktop_file}"
