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
- [ ] Help text
- [ ] Unstow
- [ ] Restow
- [ ] Adopt (check GNU Stow help)
- [ ] Special keywords to change stow target (i.e. `@root`, `@home`)
- [ ] Windows support *maybe*

*more things will be added as they come to my mind or suggested.*
