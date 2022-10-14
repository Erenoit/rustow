# Rustow
Rustow is [GNU Stow](https://www.gnu.org/software/stow/) rewritten in [Rust](https://www.rust-lang.org/). It is neither future complete nor production ready at this point. **USE WITH CAUTION!**

## Installation
1. Clone this repository:

```sh
$ git clone https://gitlab.com/Erenoit/rustow.git
```

2. Compile:

```sh
$ cargo build --release
```

3. Move executable (`target/release/rustow`) somewhere in your `$PATH`

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
- [ ] Add man page
- [ ] Add `PKGBUILD` for [AUR](aur.archlinux.org/)

*more things will be added as they come to my mind or suggested.*
