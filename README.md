# dotfiles-teto

Mi setup de Arch Linux con tema **Kasane Teto** 兎

- **Sistema:** Arch Linux, btrfs, GRUB
- **Escritorio:** Hyprland + Caelestia (Quickshell)
- **Terminal:** foot + fish + starship (colores Teto)
- **Info del sistema:** fastfetch con logo de Teto
- **Snapshots:** snapper + snap-pac + grub-btrfs

## Carpetas principales

| Carpeta | Qué tiene |
|---|---|
| `hypr/` | config de Hyprland y atajos |
| `caelestia/` | ajustes de Caelestia |
| `quickshell/` | la shell (barra, dashboard, etc.) |
| `fish/` | config de fish y saludo |
| `fastfetch/` | logo de Teto sin fondo |
| `starship.toml` | prompt con colores Teto |
| `plymouth-teto/` | pantalla de arranque |
| `teto-install/` | listas de paquetes y script de instalación |

## Instalar en otro PC con Arch

    git clone git@github.com:Ranijuan/dotfiles-teto.git ~/dotfiles-teto
    ~/dotfiles-teto/teto-install/install.sh

El script instala los paquetes de pacman, AUR y flatpak. Revisa
`teto-install/pacman.txt` antes de usarlo: trae paquetes de mi laptop
(AMD) que pueden no servir en otro hardware. Después copia las carpetas
de config a `~/.config`.

## Atajos útiles

- `SUPER + F12`: ir al workspace 1 (fondo)
- `SUPER + CTRL + F12`: volver al workspace anterior
- `SUPER + \`: abrir Nexus (ajustes de Caelestia)
## Widget del clima

El fondo de la pestaña Weather del dashboard cambia según día/noche y si llueve (`WeatherTab.qml`).

- Imágenes: `caelestia/weather-bg/` (`dia-claro`, `dia-lluvia`, `noche-clara`, `noche-lluvia`, todas `.jpg`); cópialas a `~/.config/caelestia/weather-bg/`.
- Botón de tres rayitas junto al nombre de la ciudad (`WeatherPicker.qml`): despliega las 4 opciones y abre un selector de archivos para cambiar cada imagen con el mouse. Requiere `zenity`.
- Lluvia = códigos WMO 51-67, 80-82 y 95-99.
