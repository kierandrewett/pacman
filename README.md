# pacman

A pacman-compatible CLI wrapper for `dnf`/`yum` on RPM-based Linux systems. Use the Arch Linux package manager syntax you know, and it'll do the right thing.

## Install

```sh
just install
```

Requires [just](https://github.com/casey/just). Builds a release binary and installs it to `/usr/local/bin/pacman`.

## Usage

### Install packages

```sh
sudo pacman -Sybau <package(s)>
```

| Flag | Meaning |
|------|---------|
| `-S` | sync — operate on the package database |
| `-y` | yes — assume yes / sync repos before acting |
| `-b` | best — prefer the best (latest) available version |
| `-a` | all — apply to all packages, not just listed ones |
| `-u` | upgrade — upgrade out-of-date packages |

### Update all packages

```sh
sudo pacman -Sybau
```

### Other common commands

| pacman | dnf equivalent |
|--------|----------------|
| `pacman -Ss <term>` | `dnf search <term>` |
| `pacman -Si <pkg>` | `dnf info <pkg>` |
| `pacman -Sc` | `dnf clean packages` |
| `pacman -Scc` | `dnf clean all` |
| `sudo pacman -R <pkg(s)>` | `dnf remove <pkg(s)>` |
| `sudo pacman -Rs <pkg(s)>` | `dnf remove <pkg(s)> && dnf autoremove` |
| `sudo pacman -Rsc` | `dnf autoremove` |
| `pacman -Q` | `rpm -qa` |
| `pacman -Qs <term>` | `rpm -qa \| grep <term>` |
| `pacman -Qi <pkg>` | `rpm -qi <pkg>` |
| `pacman -Ql <pkg>` | `rpm -ql <pkg>` |
| `pacman -Qo <file>` | `rpm -qf <file>` |
| `pacman -Qu` | `dnf check-update` |
| `pacman -Qe` | `dnf history userinstalled` |
| `pacman -Fs <file>` | `dnf repoquery --file <file>` |
| `pacman -Fl <pkg>` | `dnf repoquery -l <pkg>` |

Run `pacman -h` for the full help.

## Uninstall

```sh
just uninstall
```
