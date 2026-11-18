import subprocess, os, json, platform
from pathlib import Path

def run(cmd):
    try:
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.DEVNULL, text=True).strip()
    except:
        return "N/A"

r = {}

# Sistema base
r["os"] = run("cat /etc/os-release | grep PRETTY_NAME")
r["kernel"] = run("uname -r")
r["uptime"] = run("uptime -p")
r["cpu"] = run("grep 'model name' /proc/cpuinfo | head -1")
r["ram_total"] = run("free -h | awk '/Mem/{print $2}'")
r["ram_used"] = run("free -h | awk '/Mem/{print $3}'")
r["swap"] = run("free -h | awk '/Swap/{print $2,$3}'")
r["disk"] = run("df -h / | tail -1")
r["gpu"] = run("lspci | grep -i vga")

# Servicios fallidos
r["failed_services"] = run("systemctl --user list-units --state=failed --no-legend")
r["failed_system"] = run("systemctl list-units --state=failed --no-legend")

# Paquetes
r["pacman_explicit"] = run("pacman -Qe | wc -l")
r["pacman_orphans"] = run("pacman -Qtdq 2>/dev/null || echo 'ninguno'")
r["aur_packages"] = run("pacman -Qm | wc -l")

# Hyprland
r["hyprland_version"] = run("hyprctl version 2>/dev/null | head -1")
r["hyprland_monitors"] = run("hyprctl monitors -j 2>/dev/null")
r["hyprland_workspaces"] = run("hyprctl workspaces -j 2>/dev/null | python3 -c 'import json,sys; ws=json.load(sys.stdin); print(len(ws),\"workspaces\")'")

# Caelestia/Quickshell
r["qs_running"] = run("pgrep -a qs")
r["hyprexpose_running"] = run("pgrep -a hyprexpose")
r["caelestia_files"] = run("ls ~/.config/caelestia/")
r["caelestia_modules"] = run("ls ~/.config/quickshell/caelestia/modules/")

# Config files presentes
configs = [
    "~/.config/caelestia/hypr-vars.lua",
    "~/.config/caelestia/hypr-user.conf",
    "~/.config/caelestia/hypr-user.lua",