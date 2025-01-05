# Dragon-shell

Dragon-shell is a powerful, modern, and customizable shell environment built in Rust. It features intelligent auto-suggestions, plugin support, and enhanced security, making it a great choice for developers and power users alike.

---

## Features

- **Custom Commands**: Includes built-in commands like `dragon-help`, `dragon-version`, `echo`, `cd`, `lf`, and more.
- **Alias Support**: Easily create command aliases for quick access.
- **Plugin System**: Load and manage external plugins to extend the shell's capabilities.
- **History Management**: Persistent command history with file-backed storage.
- **Auto-Completion**: Intelligent suggestions for commands, environment variables, and paths.
- **Theming**: Customizable prompt themes to personalize your shell.
- **Secure Input Sanitization**: Protects against harmful input patterns.

---

## Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/your-username/dragon-shell.git
   cd dragon-shell
   ```

2. **Build the Project**
   Ensure you have [Rust installed](https://www.rust-lang.org/tools/install), then run:
   ```bash
   cargo build --release
   ```

3. **Run Dragon-shell**
   ```bash
   ./target/release/dragon-shell
   ```

---

## Usage

Once launched, Dragon-shell provides an interactive command-line interface. Here are a few examples of how to use it:

### Built-in Commands

- `dragon-help`: Display the help message for Dragon-shell.
- `dragon-version`: Show the current version of Dragon-shell.
- `echo [message]`: Print a message to the shell.
- `cd [path]`: Change the current working directory.
- `lf [path]`: List files in a directory.

### Alias Management

Create aliases in the `dragon-config.toml` file:
```toml
[aliases]
name = "list"
command = "lf"
```
Use the alias directly:
```bash
list /path/to/directory
```

### Plugins

Load external plugins with the `plugin` command:
```bash
dragon-plugin /path/to/plugin.so
```
List loaded plugins:
```bash
plugin-list
```
Unload a plugin:
```bash
plugin-unload /path/to/plugin.so
```

---

## Configuration

Dragon-shell reads its configuration from `dragon-config.toml`. If the file doesn't exist, it will be created automatically with default values.

### Default Configuration
```toml
theme = "dark"

[[aliases]]
name = "list"
command = "lf"

[[env]]
key = "EDITOR"
value = "vim"
```

You can modify this file to customize your experience.

---

## Development

### Prerequisites
- Install Rust and Cargo: [Get Started with Rust](https://www.rust-lang.org/tools/install)
- Install dependencies:
  ```bash
  cargo install reedline prettytable
  ```

### Run Locally
```bash
cargo run
```

### Testing
Dragon-shell uses `cargo test` for testing:
```bash
cargo test
```

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Acknowledgments

- [Reedline](https://github.com/nushell/reedline) - Command-line interface library.
- [PrettyTable](https://docs.rs/prettytable-rs/latest/prettytable/) - Table formatting library.
- [Serde](https://serde.rs/) - Serialization and deserialization library.
- ChatGPT - For assistance with documentation and development suggestions.

---

## Author

**Jovin Paul J**

- Email: [jovinpaulj@gmail.com](mailto:jovinpaulj@gmail.com)
- GitHub: [https://github.com/JovinPaul-J](https://github.com/JovinPaul-J)
- LinkedIn: [https://www.linkedin.com/in/jovin-paul-j-772658324](https://www.linkedin.com/in/jovin-paul-j-772658324)

