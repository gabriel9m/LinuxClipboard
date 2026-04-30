#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="${HOME}/.local/bin"
autostart_dir="${HOME}/.config/autostart"
desktop_file="${autostart_dir}/linuxclipboard.desktop"
binary_path="${bin_dir}/linuxclipboard"
data_dir="${XDG_DATA_HOME:-${HOME}/.local/share}/clipboard-history"
images_dir="${data_dir}/images"
history_path="${data_dir}/history.json"

cd "${project_dir}"

cargo build --release --features desktop-gtk --bin linuxclipboard

install -d -m 0755 "${bin_dir}"
install -d -m 0700 "${autostart_dir}" "${data_dir}" "${images_dir}"
install -m 0755 "${project_dir}/target/release/linuxclipboard" "${binary_path}"

install -m 0600 /dev/null "${desktop_file}"
cat > "${desktop_file}" <<DESKTOP
[Desktop Entry]
Type=Application
Name=LinuxClipboard
Comment=Clipboard history daemon
Exec=${binary_path}
Terminal=false
X-GNOME-Autostart-enabled=true
DESKTOP
chmod 0600 "${desktop_file}"

chmod 0700 "${data_dir}" "${images_dir}"
if [[ -f "${history_path}" ]]; then
    chmod 0600 "${history_path}"
fi
find "${images_dir}" -maxdepth 1 -type f -exec chmod 0600 {} +

echo "Installed ${binary_path}"
echo "Created autostart entry ${desktop_file}"
echo
echo "Use this command for the Super+V shortcut:"
echo "${binary_path} --toggle-popup"
echo
echo "Start/restart now with:"
echo "${binary_path} --quit || true"
echo "setsid ${binary_path} >/tmp/linuxclipboard.log 2>&1 < /dev/null &"
