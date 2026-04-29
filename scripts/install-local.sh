#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="${HOME}/.local/bin"
autostart_dir="${HOME}/.config/autostart"
desktop_file="${autostart_dir}/linuxclipboard.desktop"
binary_path="${bin_dir}/linuxclipboard"

cd "${project_dir}"

cargo build --release --features desktop-gtk --bin linuxclipboard

install -d "${bin_dir}" "${autostart_dir}"
install -m 0755 "${project_dir}/target/release/linuxclipboard" "${binary_path}"

cat > "${desktop_file}" <<DESKTOP
[Desktop Entry]
Type=Application
Name=LinuxClipboard
Comment=Clipboard history daemon
Exec=${binary_path}
Terminal=false
X-GNOME-Autostart-enabled=true
DESKTOP

echo "Installed ${binary_path}"
echo "Created autostart entry ${desktop_file}"
echo
echo "Use this command for the Super+V shortcut:"
echo "${binary_path} --toggle-popup"
echo
echo "Start/restart now with:"
echo "${binary_path} --quit || true"
echo "setsid ${binary_path} >/tmp/linuxclipboard.log 2>&1 < /dev/null &"
