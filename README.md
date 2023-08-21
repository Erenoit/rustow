# Rustow

Rustow is [GNU Stow] rewritten in [Rust]. While writing Rustow, [GNU Stow] only used as a starting
idea not as an objective; so expect some different behavior and/or missing features. Rustow also has
some additional features over [GNU Stow]. Check man page for more details. It may have some bugs.
**USE WITH CAUTION!**

## Installation

### Arch Linux

Rustow is available in [AUR] as [rustow-git]. Use your favorite AUR Helper to install. Examples:

- [Aura]

```sh
$ aura -A rustow-git
```

- [Yay]

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

## Dependencies

It only depends on `libc` at runtime to be able to communicate with the OS.

Also, it depends on `cargo` to be built.

[GNU Stow]: https://www.gnu.org/software/stow/
[Rust]: https://www.rust-lang.org/
[AUR]: aur.archlinux.org/
[rustow-git]: https://aur.archlinux.org/packages/rustow-git
[Aura]: https://github.com/fosskers/aura
[Yay]: https://github.com/Jguer/yay
