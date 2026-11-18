import subprocess, os, json
from pathlib import Path

def run(cmd):
    try:
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.DEVNULL, text=True).strip()
    except:
        return "N/A"

r = {}
r["os"] = run("cat /etc/os-release | grep PRETTY_NAME")
r["kernel"] = run("uname -r")
r["uptime"] = run("uptime -p")
r["cpu"] = run("grep 'model name' /proc/cpuinfo | head -1")
r["ram_total"] = run("free -h | awk '/Mem/{print $2}'")
r["ram_used"] = run("free -h | awk '/Mem/{print $3}'")
r["swap"] = run("free -h | awk '/Swap/{print $2,$3}'")
r["disk"] = run("df -h / | tail -1")
r["gpu"] = run("lspci | grep -i vga")
r["failed_services"] = run("systemctl --user list-units --state=failed --no-legend")
r["failed_system"] = run("systemctl list-units --state=failed --no-legend")
r["pacman_explicit"] = run("pacman -Qe | wc -l")
r["pacman_orphans"] = run("pacman -Qtdq 2>/dev/null || echo 'ninguno'")
r["aur_packages"] = run("pacman -Qm | wc -l")
r["hyprland_version"] = run("hyprctl version 2>/dev/null | head -1")
r["hyprland_monitors"] = run("hyprctl monitors 2>/dev/null")
r["qs_running"] = run("pgrep -a qs")
r["hyprexpose_running"] = run("pgrep -a hyprexpose")
r["caelestia_files"] = run("ls ~/.config/caelestia/")
r["caelestia_modules"] = run("ls ~/.config/quickshell/caelestia/modules/")
configs = [
    "~/.config/caelestia/hypr-vars.lua",
    "~/.config/caelestia/hypr-user.conf",
    "~/.config/caelestia/hypr-user.lua",
    "~/.config/caelestia/shell.json",
    "~/.config/fish/config.fish",
    "~/.config/hypr/variables.lua",
    "~/.config/hypr/hyprland/keybinds.lua",
    "/usr/share/plymouth/themes/teto/teto.script",
    "~/.local/bin/cursor-contrast.sh",
    "/home/ranijuan/ajustes-hypr.py",
    "/home/ranijuan/desinstalador.py",
]
r["config_files"] = {c: "ok" if Path(os.path.expanduser(c)).exists() else "FALTA" for c in configs}
r["git_status"] = run("git -C ~/.config status --short")
r["git_remote"] = run("git -C ~/.config remote -v")
r["audio"] = run("pactl info | grep 'Server Name'")
r["cava"] = run("which cava")
r["autostart"] = run("cat ~/.config/caelestia/hypr-user.conf")
r["keybinds_custom"] = run("grep -n 'kbCustom\\|cmdCustom\\|kbMonitor\\|kbWallpaper\\|kbWindowCycle' ~/.config/caelestia/hypr-vars.lua ~/.config/hypr/variables.lua 2>/dev/null")
print(json.dumps(r, indent=2, ensure_ascii=False))