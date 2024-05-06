# Installation

### Cargo (crates.io)

![Crates.io](https://img.shields.io/crates/v/idid?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fidid)

If you already have a Rust environment set up, you can use the `cargo install` command:

    cargo install idid

Cargo will build the `idid` binary and place it in `$HOME/.cargo/bin/idid`.

### Cargo (git)

If you already have a Rust environment set up, you can use the `cargo install` command in your local clone of the repo:

    git clone https://github.com/jidn/idid-rust.git
    cd idid-rust
    cargo install --path .

Cargo will build the `idid` binary and place it in `$HOME/.cargo/bin`.

### Build from repository

To build idid, follow these steps:

Clone the repository:

```sh
git clone https://github.com/jidn/idid-rust.git
```

Navigate to the project directory:

```sh
cd idid-rust
```

Build the project:

```sh
cargo build --release
```

Now add the executable to a PATH or add the directory to your path:

```sh
export PATH="$PATH:/path/to/idid-rust/target/release/"
```

For a more permanent solution, [ask Google](https://www.google.com/search?q=linux+add+directory+to+path)

### Arch Linux

1. Ensure both `rust` and `cargo` are installed.
2. `cd scripts`
3. `makepkg --install`

Uninstall is `pacman -R idid`.
