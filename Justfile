install:
    cargo build --release
    sudo install -m 755 target/release/pacman /usr/local/bin/pacman

uninstall:
    sudo rm -f /usr/local/bin/pacman

reinstall: uninstall install
