use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use dirs_next::home_dir;  // Correctly importing home_dir

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add { description: String },
    List,
    Complete { id: usize },
    Remove { id: usize },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: usize,
    description: String,
    completed: bool,
}

struct TodoManager {
    todos: Vec<Todo>,
    file_path: PathBuf,
}

impl TodoManager {
    fn new() -> Result<Self, io::Error> {
        let file_path = home_dir()  // Correct usage of home_dir
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find home directory"))?
            .join(".rust_todos.json");

        let todos = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Ok(TodoManager { todos, file_path })
    }

    fn save(&self) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(&self.todos)?;
        let mut file = File::create(&self.file_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn add(&mut self, description: String) {
        let new_id = self.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let todo = Todo {
            id: new_id,
            description,
            completed: false,
        };
        self.todos.push(todo);
        if let Err(err) = self.save() {
            eprintln!("Failed to save todo: {}", err);
        } else {
            println!("Todo added successfully!");
        }
    }

    fn list(&self) {
        if self.todos.is_empty() {
            println!("No todos found.");
            return;
        }

        println!("{:<5} {:<10} {}", "ID", "Status", "Description");
        println!("{}", "-".repeat(30));
        for todo in &self.todos {
            let status = if todo.completed { "[x]" } else { "[ ]" };
            println!("{:<5} {:<10} {}", todo.id, status, todo.description);
        }
    }

    fn complete(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = true;
            if let Err(err) = self.save() {
                eprintln!("Failed to save changes: {}", err);
            } else {
                println!("Todo {} marked as completed!", id);
            }
        } else {
            println!("Todo with id {} not found.", id);
        }
    }

    fn remove(&mut self, id: usize) {
        if let Some(pos) = self.todos.iter().position(|t| t.id == id) {
            self.todos.remove(pos);
            if let Err(err) = self.save() {
                eprintln!("Failed to save changes: {}", err);
            } else {
                println!("Todo {} removed successfully!", id);
            }
        } else {
            println!("Todo with id {} not found.", id);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut manager = match TodoManager::new() {
        Ok(manager) => manager,
        Err(err) => {
            eprintln!("Error initializing TodoManager: {}", err);
            return;
        }
    };

    match &cli.command {
        Some(Commands::Add { description }) => {
            manager.add(description.to_string());
        }
        Some(Commands::List) => {
            manager.list();
        }
        Some(Commands::Complete { id }) => {
            manager.complete(*id);
        }
        Some(Commands::Remove { id }) => {
            manager.remove(*id);
        }
        None => {
            println!("No command specified. Use --help to see available commands.");
        }
    }
}

