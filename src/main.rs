use std::env;
use std::os::unix::process::CommandExt;
use std::process::{self, Command, Stdio};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the first package manager found: dnf, then yum.
fn find_package_manager() -> &'static str {
    let has = |name: &str| {
        Command::new("which")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    if has("dnf") {
        "dnf"
    } else if has("yum") {
        "yum"
    } else {
        eprintln!("error: neither dnf nor yum found in PATH");
        process::exit(1);
    }
}

/// Replace the current process with `pm args…`.
/// Never returns on success.
fn exec_cmd(_privileged: bool, pm: &str, args: &[String]) -> ! {
    eprintln!("+ {pm} {}", args.join(" "));
    let err = Command::new(pm).args(args).exec();
    eprintln!("error: failed to exec {pm}: {err}");
    process::exit(1);
}


fn print_help() {
    println!(
        "pacman v{VERSION} — pacman-compatible wrapper for dnf/yum\n\
         \n\
         usage: pacman <operation> [options] [targets]\n\
         \n\
         FLAG MEANINGS\n\
         \x20 -S  sync          operate on the package database\n\
         \x20 -y  yes           assume yes / sync repos before acting\n\
         \x20 -b  best          prefer the best (latest) available version\n\
         \x20 -a  all           apply to all packages, not just listed ones\n\
         \x20 -u  upgrade       upgrade out-of-date packages\n\
         \n\
         COMMON USAGE\n\
         \x20 pacman -Sybau <pkg(s)>   install packages        → dnf install <pkg(s)>\n\
         \x20 pacman -Sybau            update all packages     → dnf update\n\
         \x20 pacman -Ss <term>        search packages         → dnf search <term>\n\
         \x20 pacman -Si <pkg>         show package info       → dnf info <pkg>\n\
         \x20 pacman -Sc / -Scc        clean package cache     → dnf clean packages/all\n\
         \n\
         REMOVE (-R)\n\
         \x20 pacman -R  <pkg(s)>      remove packages         → dnf remove <pkg(s)>\n\
         \x20 pacman -Rs <pkg(s)>      remove + orphan sweep   → dnf remove + autoremove\n\
         \x20 pacman -Rsc              remove all orphans      → dnf autoremove\n\
         \n\
         QUERY (-Q)\n\
         \x20 pacman -Q                list all installed      → rpm -qa\n\
         \x20 pacman -Qs <term>        search installed        → rpm -qa | grep\n\
         \x20 pacman -Qi <pkg>         show installed info     → rpm -qi\n\
         \x20 pacman -Ql <pkg>         list package files      → rpm -ql\n\
         \x20 pacman -Qo <file>        find file owner         → rpm -qf\n\
         \x20 pacman -Qu               list upgradeable        → dnf check-update\n\
         \x20 pacman -Qe               explicitly installed    → dnf history userinstalled\n\
         \n\
         FILES (-F)\n\
         \x20 pacman -Fs <file>        which package owns file → dnf repoquery --file\n\
         \x20 pacman -Fl <pkg>         list files in pkg       → dnf repoquery -l\n\
         \n\
         GLOBAL OPTIONS\n\
         \x20 --noconfirm              skip confirmation prompts\n"
    );
}

/// Count how many times flag `f` appears in the flags slice.
fn count_flag(flags: &[char], f: char) -> usize {
    flags.iter().filter(|&&c| c == f).count()
}

fn main() {
    let raw: Vec<String> = env::args().collect();

    if raw.len() < 2 {
        print_help();
        return;
    }

    // Global flags that may appear anywhere
    let noconfirm = raw.iter().any(|a| a == "--noconfirm");
    let needed = raw.iter().any(|a| a == "--needed");

    let yes = if noconfirm { vec!["-y".to_string()] } else { vec![] };

    let op_str = &raw[1];

    match op_str.as_str() {
        "-h" | "--help" => { print_help(); return; }
        "-V" | "--version" => { println!("pacman v{VERSION}"); return; }
        _ => {}
    }

    if !op_str.starts_with('-') {
        eprintln!("error: no operation specified (use -h for help)");
        process::exit(1);
    }

    // Parse "-Syu" → operation='S', flags=['y','u']
    // Parse "--sync" → operation='S', extra flags from subsequent "--foo" args
    let (operation, flags): (char, Vec<char>) = if op_str.starts_with("--") {
        let op = match op_str.as_str() {
            "--sync"     => 'S',
            "--remove"   => 'R',
            "--upgrade"  => 'U',
            "--query"    => 'Q',
            "--files"    => 'F',
            "--database" => 'D',
            _ => { eprintln!("error: unknown operation '{op_str}' (use -h for help)"); process::exit(1); }
        };
        let extra: Vec<char> = raw[2..].iter()
            .filter(|a| a.starts_with("--"))
            .filter_map(|a| match a.as_str() {
                "--search"       | "--recursive"  => Some('s'),
                "--info"                          => Some('i'),
                "--refresh"                       => Some('y'),
                "--sysupgrade"   | "--upgrades"   => Some('u'),
                "--clean"                         => Some('c'),
                "--nosave"                        => Some('n'),
                "--list"                          => Some('l'),
                "--owns"                          => Some('o'),
                "--explicit"                      => Some('e'),
                "--downloadonly"                  => Some('w'),
                "--groups"                        => Some('g'),
                _ => None,
            })
            .collect();
        (op, extra)
    } else {
        // Short form: strip '-', first char = operation, rest = modifier flags
        let chars: Vec<char> = op_str.chars().skip(1).collect();
        if chars.is_empty() {
            eprintln!("error: no operation specified (use -h for help)");
            process::exit(1);
        }
        (chars[0], chars[1..].to_vec())
    };

    // Remaining positional arguments (not starting with '-')
    let packages: Vec<String> = raw[2..]
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    let has = |f: char| flags.contains(&f);

    let pm = find_package_manager();

    match operation {
        // ── SYNC ─────────────────────────────────────────────────────────────
        'S' => {
            if !packages.is_empty() && !has('s') && !has('i') && !has('w') && !has('g') {
                // -S <pkg(s)>  install — packages always win regardless of other flags
                // e.g. -Sybau pkg, -Syu pkg, -S pkg all install
                let mut a = vec!["install".to_string()];
                a.extend(yes);
                if needed { a.push("--setopt=obsoletes=false".to_string()); }
                a.extend(packages.iter().cloned());
                exec_cmd(true, pm, &a);
            } else if has('s') {
                // -Ss <term>  search available packages
                let mut a = vec!["search".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, pm, &a);
            } else if has('i') {
                // -Si [pkg]  info
                let mut a = vec!["info".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, pm, &a);
            } else if count_flag(&flags, 'c') >= 2 {
                // -Scc  clean all
                exec_cmd(true, pm, &["clean".to_string(), "all".to_string()]);
            } else if has('c') {
                // -Sc  clean packages
                exec_cmd(true, pm, &["clean".to_string(), "packages".to_string()]);
            } else if has('w') {
                // -Sw <pkg>  download only
                let mut a = vec!["download".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, pm, &a);
            } else if has('g') {
                // -Sg [group]  list groups / contents
                let mut a = vec!["group".to_string(), "list".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, pm, &a);
            } else if has('u') {
                // -Sybau / -Syu / -Su  update all packages
                let mut a = vec!["update".to_string()];
                a.extend(yes);
                exec_cmd(true, pm, &a);
            } else if has('y') {
                // -Sy  refresh package databases
                exec_cmd(true, pm, &["makecache".to_string()]);
            } else {
                eprintln!("error: no targets specified (use -h for help)");
                process::exit(1);
            }
        }

        // ── REMOVE ────────────────────────────────────────────────────────────
        'R' => {
            if packages.is_empty() && !has('s') {
                eprintln!("error: no targets specified (use -h for help)");
                process::exit(1);
            }
            if packages.is_empty() && has('s') {
                // -Rsc  remove all orphans
                let mut a = vec!["autoremove".to_string()];
                a.extend(yes);
                exec_cmd(true, pm, &a);
            } else {
                // -R / -Rs / -Rns  remove packages
                // dnf remove already cleans up deps; run autoremove afterwards if -s
                let mut a = vec!["remove".to_string()];
                a.extend(yes.iter().cloned());
                a.extend(packages.iter().cloned());

                if has('s') {
                    // Run remove then autoremove: chain two commands via sh -c
                    let pm_y = yes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
                    let pkgs = packages.join(" ");
                    let cmd = format!("{pm} remove {pm_y} {pkgs} && {pm} autoremove {pm_y}");
                    let err = Command::new("sudo").args(["sh", "-c", &cmd]).exec();
                    eprintln!("error: failed to exec sudo: {err}");
                    process::exit(1);
                } else {
                    exec_cmd(true, pm, &a);
                }
            }
        }

        // ── UPGRADE LOCAL FILE ────────────────────────────────────────────────
        'U' => {
            if packages.is_empty() {
                eprintln!("error: no targets specified (use -h for help)");
                process::exit(1);
            }
            let mut a = vec!["install".to_string()];
            a.extend(yes);
            a.extend(packages.iter().cloned());
            exec_cmd(true, pm, &a);
        }

        // ── QUERY ─────────────────────────────────────────────────────────────
        'Q' => {
            if has('s') {
                // -Qs [term]  search installed packages
                if packages.is_empty() {
                    exec_cmd(false, "rpm", &["-qa".to_string()]);
                } else {
                    // rpm -qa piped through grep — use sh -c for simplicity
                    let term = &packages[0];
                    let cmd = format!("rpm -qa --qf '%{{name}} %{{version}}-%{{release}}\\n' | grep -i '{term}'");
                    let err = Command::new("sh").args(["-c", &cmd]).exec();
                    eprintln!("error: {err}");
                    process::exit(1);
                }
            } else if has('i') {
                // -Qi <pkg>  show package info
                if let Some(pkg) = packages.first() {
                    exec_cmd(false, "rpm", &["-qi".to_string(), pkg.clone()]);
                } else {
                    exec_cmd(false, "rpm", &["-qa".to_string()]);
                }
            } else if has('l') {
                // -Ql <pkg>  list package files
                if let Some(pkg) = packages.first() {
                    exec_cmd(false, "rpm", &["-ql".to_string(), pkg.clone()]);
                } else {
                    eprintln!("error: no targets specified");
                    process::exit(1);
                }
            } else if has('o') {
                // -Qo <file>  find which package owns a file
                if let Some(file) = packages.first() {
                    exec_cmd(false, "rpm", &["-qf".to_string(), file.clone()]);
                } else {
                    eprintln!("error: no targets specified");
                    process::exit(1);
                }
            } else if has('u') {
                // -Qu  list upgradeable packages
                exec_cmd(false, pm, &["check-update".to_string()]);
            } else if has('e') {
                // -Qe  explicitly installed packages
                exec_cmd(false, pm, &["history".to_string(), "userinstalled".to_string()]);
            } else if !packages.is_empty() {
                // -Q <pkg>  check if specific package is installed
                let mut a = vec!["-q".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, "rpm", &a);
            } else {
                // -Q  list all installed packages
                exec_cmd(false, "rpm", &["-qa".to_string()]);
            }
        }

        // ── FILES ─────────────────────────────────────────────────────────────
        'F' => {
            if has('s') || (!packages.is_empty() && !has('l')) {
                // -Fs <file>  which package in repos provides this file
                let mut a = vec!["repoquery".to_string(), "--file".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, pm, &a);
            } else if has('l') {
                // -Fl <pkg>  list files owned by a repo package
                let mut a = vec!["repoquery".to_string(), "-l".to_string()];
                a.extend(packages.iter().cloned());
                exec_cmd(false, pm, &a);
            } else {
                eprintln!("error: no targets specified (use -h for help)");
                process::exit(1);
            }
        }

        // ── DATABASE ──────────────────────────────────────────────────────────
        'D' => {
            eprintln!("warning: -D (database) operations are not supported in the dnf/yum wrapper");
            process::exit(1);
        }

        _ => {
            eprintln!("error: invalid operation '{operation}' (use -h for help)");
            process::exit(1);
        }
    }
}
