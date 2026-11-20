#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"

if ! command -v pacman >/dev/null 2>&1; then
  echo "Este script es solo para Arch Linux."
  exit 1
fi

echo "==> 1/4 Paquetes de pacman"
sudo pacman -S --needed - < "$DIR/pacman.txt"

echo "==> 2/4 Paquetes de AUR"
if ! command -v yay >/dev/null 2>&1; then
  sudo pacman -S --needed git base-devel
  tmp="$(mktemp -d)"
  git clone https://aur.archlinux.org/yay-bin.git "$tmp/yay-bin"
  (cd "$tmp/yay-bin" && makepkg -si)
fi
xargs -a "$DIR/aur.txt" yay -S --needed

echo "==> 3/4 Flatpak"
if command -v flatpak >/dev/null 2>&1; then
  flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
  xargs -a "$DIR/flatpak.txt" -r flatpak install -y flathub
fi

echo "==> 4/4 Listo"
echo "Siguiente paso: copia tu repo de dotfiles a ~/.config y reinicia."
echo "Para usar fish: chsh -s /usr/bin/fish"