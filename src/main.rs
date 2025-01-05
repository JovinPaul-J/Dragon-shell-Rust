use reedline::{Reedline, Prompt, FileBackedHistory, Completer, Suggestion, Span, PromptEditMode, PromptHistorySearch};
use std::env;
use serde::{Deserialize, Serialize};
use std::fs;
use std::borrow::Cow;
use libloading::Library;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{Command, Stdio}; 
use prettytable::{Table, Row, Cell};
use std::path::Path;

fn main() {
    let config_path = "dragon-config.toml";
    let config = load_config(config_path);

    // Set environment variables
    for env in config.env {
        std::env::set_var(env.key, env.value);
    }

    // Handle aliases in the command handler
    let aliases = config
        .aliases
        .into_iter()
        .map(|alias| (alias.name, alias.command))
        .collect::<std::collections::HashMap<_, _>>();

    // Declare plugins as an empty vector
    let mut plugins: Vec<Library> = Vec::new();

    // Command history setup with both capacity and file path
    let history_file = PathBuf::from(".dragon_history");
    let history_capacity = 1000;  // Adjust the capacity as needed

    // Create a FileBackedHistory with the capacity and file path
    let history = match FileBackedHistory::with_file(history_capacity, history_file) {
        Ok(h) => Box::new(h),
        Err(e) => {
            eprintln!("Error setting up history: {}", e);
            return;
        }
    };

    // Auto-completion setup
    let completer = Box::new(DragonCompleter {
        commands: vec![
            "dragon-help".to_string(),
            "dragon-version".to_string(),
            "echo".to_string(),
            "cd".to_string(),
            "lf".to_string(),
            "kill".to_string(),
            "run".to_string(),
            "plugin".to_string(),
            "plugin-list".to_string(),
            "plugin-unload".to_string(),
        ],
    });

    // Reedline setup with history and auto-completion
    let mut reedline = Reedline::create()
        .with_completer(completer)
        .with_history(history);
    
        let custom_prompt = CustomPrompt::new("Dragon-shell");  // Create the prompt instance
        let prompt = &custom_prompt; // Borrow the prompt

        println!("{}", custom_prompt.render_prompt_left()); // Borrow `custom_prompt` here

    println!("Welcome to Dragon-shell!");

    // Main REPL loop
    loop {
        match reedline.read_line(prompt) {
            Ok(signal) => match signal {
                reedline::Signal::Success(line) => { // Handle the event when input is successfully received
                    let input = sanitize_input(&line.trim());
                    if input == "exit" {
                        println!("Goodbye!");
                        break;
                    }

                    let mut parts = input.split_whitespace();
                    let command = parts.next().unwrap_or("");
                    let args: Vec<&str> = parts.collect();

                    if let Some(output) = handle_dragon_command(command, &args, &aliases, &mut plugins) {
                        println!("{}", output);
                    } else if command == "run" {
                        if let Some(script_path) = args.get(0) {
                            if let Err(e) = execute_script(script_path, &aliases, &args[1..], &mut plugins) {  // Pass `plugins` here
                                println!("Error running script: {}", e);
                            }
                        } else {
                            println!("Usage: dragon-run <script_path>");
                        }                        
                    } else if command == "plugin" {
                        if let Some(plugin_path) = args.get(0) {
                            if let Ok(_) = load_plugin(plugin_path, &mut plugins) {
                                if let Some(output) = execute_plugin_command(&plugins.last().unwrap(), "plugin_main", &args[1..]) {
                                    println!("{}", output);
                                } else {
                                    println!("Failed to find the plugin function.");
                                }
                            } else {
                                println!("Failed to load plugin: {}", plugin_path);
                            }
                        } else {
                            println!("Usage: dragon-plugin <plugin_path> <command> [args...]");
                        }
                    } else {
                        if let Err(e) = execute_external_command(command, &args) {
                            println!("Error executing command '{}': {}", command, e);
                        }
                    }
                },
                reedline::Signal::CtrlC => {
                    // Handle Ctrl+C signal
                    println!("Received Ctrl+C, aborting...");
                },
                reedline::Signal::CtrlD => {
                    // Handle Ctrl+D (End of input)
                    println!("Received Ctrl+D, exiting...");
                    break;
                },
            },
            Err(e) => {
                // Handle the error returned by reedline (crossterm::ErrorKind)
                println!("Error reading input: {:?}", e);
            }
        }
    }
}

// Dragon custom command handler
fn handle_dragon_command(
    command: &str,
    args: &[&str],
    aliases: &std::collections::HashMap<String, String>,
    plugins: &mut Vec<Library>, // Accept mutable reference to Vec<Library>
) -> Option<String> {
    match command {
        "alias" => Some(aliases.iter().map(|(k, v)| format!("{} -> {}", k, v)).collect::<Vec<String>>().join("\n")),
        
        "d-help" => Some("Welcome to Dragon-shell! Custom commands include: dragon-help, dragon-version, alias, plugin-list, plugin-unload".to_string()),
        
        "d-version" => Some("Dragon-shell v1.0".to_string()),
        
        "echo" => Some(args.join(" ")),
        
        "cd" => {
            if let Some(path) = args.get(0) {
                if let Err(e) = env::set_current_dir(path) {
                    return Some(format!("Error changing directory: {}", e));
                }
                Some(format!("Changed directory to {}", path))
            } else {
                Some("Usage: dragon-cd <path>".to_string())
            }
        },
        "lf" => {
            let path = args.get(0).unwrap_or(&".");
            list_files_in_directory(path)
        },
        "kill" => {
            if let Some(pid) = args.get(0) {
                match Command::new("taskkill").arg("/PID").arg(pid).arg("/F").status() {
                    Ok(status) => {
                        if status.success() {
                            Some(format!("Process {} terminated", pid))
                        } else {
                            Some(format!("Error terminating process {}: non-zero exit status", pid))
                        }
                    }
                    Err(e) => Some(format!("Failed to terminate process: {}", e)),
                }
            } else {
                Some("Usage: dragon-kill <pid>".to_string())
            }
        },
        "plugin-list" => list_plugins(plugins),
        "plugin-unload" => {
            if let Some(plugin_path) = args.get(0) {
                if let Some(pos) = plugins.iter().position(|p| format!("{:?}", p).contains(plugin_path)) {
                    plugins.remove(pos);
                    return Some(format!("Unloaded plugin: {}", plugin_path));
                }
                return Some(format!("Plugin {} not found", plugin_path));
            }
            Some("Usage: plugin-unload <plugin_path>".to_string())
        },
        _ => None, // If not a custom command
    }
}

// Executes a script from file
fn execute_script(
    file_path: &str,
    aliases: &std::collections::HashMap<String, String>,
    script_args: &[&str],  // Script args are still references to strings
    plugins: &mut Vec<Library>
) -> io::Result<()> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        if let Ok(command_line) = line {
            let mut parts = command_line.split_whitespace();
            let command = parts.next().unwrap_or("");
            let args: Vec<&str> = parts.chain(script_args.iter().map(|&arg| arg)).collect();  // Fixed

            if let Some(output) = handle_dragon_command(command, &args, aliases, plugins) {
                println!("{}", output);
            } else {
                execute_external_command(command, &args).unwrap_or_else(|e| {
                    eprintln!("Error executing command '{}': {}", command, e);
                });
            }
        }
    }

    Ok(())
}

// Plugin loading function
type PluginCommand = fn(args: &[&str]) -> String;

fn load_plugin(plugin_path: &str, plugins: &mut Vec<Library>) -> io::Result<()> {
    unsafe {
        match Library::new(plugin_path) {
            Ok(lib) => {
                plugins.push(lib); // Store the loaded library in plugins
                Ok(())
            },
            Err(e) => Err(io::Error::new(std::io::ErrorKind::Other, format!("Failed to load plugin: {}", e))),
        }
    }
}

fn list_plugins(plugins: &Vec<Library>) -> Option<String> {
    // Create a table to display plugins
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Plugin Path"),
        Cell::new("Loaded"),
    ]));

    for plugin in plugins {
        // Here, we just display the path as a string for each plugin
        let plugin_path = format!("{:?}", plugin); // Display the plugin path (you can adjust this as needed)
        table.add_row(Row::new(vec![
            Cell::new(&plugin_path),
            Cell::new("Yes"), // Indicate plugin is loaded
        ]));
    }

    // Return the table as a string
    Some(table.to_string())
}

fn execute_plugin_command(lib: &Library, command_name: &str, args: &[&str]) -> Option<String> {
    unsafe {
        match lib.get::<PluginCommand>(command_name.as_bytes()) {
            Ok(func) => {
                println!("Executing plugin function: {}", command_name);
                Some(func(args)) // Call the plugin function with arguments
            },
            Err(_) => {
                println!("Error: Failed to find the plugin function: {}", command_name);
                None
            }
        }
    }
}

// Executes external commands
fn execute_external_command(command: &str, args: &[&str]) -> io::Result<()> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::inherit())  // Display the standard output
        .stderr(Stdio::inherit())  // Display the error output
        .stdin(Stdio::inherit())   // Inherit standard input
        .spawn()?; // Start the command

    child.wait()?; // Wait for the command to finish
    Ok(())
}

// Custom completer for Dragon-shell commands
struct DragonCompleter {
    commands: Vec<String>,
}

impl Completer for DragonCompleter {
    fn complete(&mut self, line: &str, _pos: usize) -> Vec<Suggestion> {
        // Implement file path and environment variable completion
        if line.starts_with('$') {
            return complete_env_var(line);  // Function to complete environment variables
        }
        // Command completion as already implemented
        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(line))
            .map(|cmd| Suggestion {
                value: cmd.to_string(),
                description: Some(format!("Command for: {}", cmd)),
                append_whitespace: true,
                extra: None,
                span: Span::new(0, line.len())
            })
            .collect()
    }
}

fn complete_env_var(line: &str) -> Vec<Suggestion> {
    env::vars()
        .filter(|(key, _)| key.starts_with(&line[1..]))  // Skip the $ sign
        .map(|(key, _)| Suggestion {
            value: format!("${}", key),
            description: Some(format!("Environment variable: {}", key)),
            append_whitespace: false,
            extra: None,
            span: Span::new(0, line.len())
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
struct Config {
    theme: String,
    aliases: Vec<Alias>,
    env: Vec<EnvVar>,
}

#[derive(Serialize, Deserialize)]
struct Alias {
    name: String,
    command: String,
}

#[derive(Serialize, Deserialize)]
struct EnvVar {
    key: String,
    value: String,
}

fn load_config(path: &str) -> Config {
    if !PathBuf::from(path).exists() {
        // File doesn't exist, create it with the default config
        let default_config = Config {
            theme: "dark".to_string(),
            aliases: vec![],
            env: vec![],
        };
        let toml_string = toml::to_string(&default_config).unwrap();
        fs::write(path, toml_string).unwrap();
        return default_config;
    }

    // If file exists, load the configuration
    let content = fs::read_to_string(path).expect("Failed to read the config file");
    toml::from_str(&content).expect("Failed to parse the config file")
}
fn sanitize_input(input: &str) -> String {
    // Remove dangerous characters or patterns
    input.replace(";", "").replace("&", "").replace("||", "")
}

struct CustomPrompt {
    theme: String,
}

impl CustomPrompt {
    // Define a constructor for CustomPrompt
    fn new(theme: &str) -> Self {
        CustomPrompt {
            theme: theme.to_string(),
        }
    }
}

impl Prompt for CustomPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        // Implement a left prompt (you can customize this further)
        Cow::Borrowed(&self.theme)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        // You can also customize the right side of the prompt
        Cow::Borrowed(" ")
    }

    fn render_prompt_indicator(&self, _mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("> ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("...> ")
    }

    fn render_prompt_history_search_indicator(&self, _: PromptHistorySearch) -> Cow<'_, str> {
        Cow::Borrowed("??> ")
    }
}

fn list_files_in_directory(path: &str) -> Option<String> {
    let path = Path::new(path);
    if path.exists() && path.is_dir() {
        let entries = fs::read_dir(path).ok()?;

        // Create a table to display files
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("File Name"),
            Cell::new("Size (bytes)"),
            Cell::new("Modified Date"),
        ]));
        for entry in entries.filter_map(Result::ok) {
            let metadata = entry.metadata().ok()?;
            let file_name = entry.file_name().to_string_lossy().into_owned();
            let file_size = metadata.len().to_string();

            table.add_row(Row::new(vec![
                Cell::new(&file_name),
                Cell::new(&file_size),
            ]));
        }
        // Return the table as a string
        Some(table.to_string())
    } else {
        Some(format!("Error: '{}' is not a valid directory", path.display()))  // Use display() here
    }
}