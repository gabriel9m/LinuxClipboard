#!/usr/bin/env bash
set -euo pipefail

binary_path="${HOME}/.local/bin/linuxclipboard"
desktop_file="${HOME}/.config/autostart/linuxclipboard.desktop"
data_dir="${XDG_DATA_HOME:-${HOME}/.local/share}/clipboard-history"
purge_data=false

for arg in "$@"; do
    case "${arg}" in
        --purge-data)
            purge_data=true
            ;;
        *)
            echo "Unknown argument: ${arg}" >&2
            echo "Usage: $0 [--purge-data]" >&2
            exit 2
            ;;
    esac
done

if [[ -x "${binary_path}" ]]; then
    "${binary_path}" --quit || true
fi

rm -f "${binary_path}" "${desktop_file}"

echo "Removed ${binary_path}"
echo "Removed ${desktop_file}"

if [[ "${purge_data}" == true ]]; then
    rm -rf "${data_dir}"
    echo "Removed ${data_dir}"
else
    echo "Preserved ${data_dir}"
    echo "Run '$0 --purge-data' to remove persisted history and images."
fi
