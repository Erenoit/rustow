# Rustow
Rustow is [GNU Stow](https://www.gnu.org/software/stow/) rewritten in [Rust](https://www.rust-lang.org/). It may have some bugs. **USE WITH CAUTION!**

## Installation

### Arch Linux
Rustow is available in [AUR](aur.archlinux.org/) as [rustow-git](https://aur.archlinux.org/packages/rustow-git). Use your favorite AUR Helper to install.
Examples:

- [Aura](https://github.com/fosskers/aura)

```sh
$ aura -A rustow-git
```

- [Yay](https://github.com/Jguer/yay)

```sh
$ yay -S rustow-git
```

### Others
Rustow is not in any official repository (as far as I know). So, you should build it from source.
1. Clone this repository:

```sh
$ git clone https://gitlab.com/Erenoit/rustow.git
```

2. Compile:

```sh
$ cargo build --release
```

3. Move executable (`target/release/rustow`) somewhere in your `$PATH`:

```sh
$ install -Dm755 target/release/rustow /usr/bin/rustow
$ install -Dm644 LICENSE /usr/share/licenses/rustow/LICENSE
$ install -Dm644 rustow.1 /usr/share/man/man1/rustow.1
```

## TODO
- [x] Basic functionality
- [x] Better stow algorithm (i.e. if file already exist program fails)
- [x] Take arguments for choosing what to stow
- [x] Help text
- [x] Unstow
- [x] Restow
- [x] Adopt (check GNU Stow help)
- [x] `--stow-dir` and `--target-dir` options
- [x] Special keywords to change stow target (i.e. `@root`, `@home`)
    - [x] `@home`
    - [x] `@root` *(may need root privileges)*
- [x] Add tests
- [x] ~~Windows support *maybe*~~ **CANCELED**
- [x] Handle errors instead of using `_ = func();`
- [x] `--version` flag
- [x] `--verbose` flag
- [x] `--no-special-keywords` flag
- [x] `--no-security-check` flag
- [x] `--simulate` flag
- [x] Check everything inside `@root` before stow (security check)
- [x] Add man page
- [x] Add `PKGBUILD` for [AUR](aur.archlinux.org/)
- [x] Upload package to [AUR](aur.archlinux.org/)
- [ ] TUI application

*more things will be added as they come to my mind or suggested.*
