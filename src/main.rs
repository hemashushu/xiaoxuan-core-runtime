// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run an application
    Run {
        /// Path to application
        application_path: Option<String>,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Create a new module
    New {
        /// Module type
        #[arg(short, long)]
        #[arg(value_enum)]
        #[arg(default_value = "app")]
        type_: NewModuleType,

        /// Module name
        module_name: String,

        /// Directory where the created module is stored
        location: Option<PathBuf>,
    },
    /// Run unit tests
    Test {
        /// Prefix of path name of the test units.
        path_name_prefix: Option<String>,

        /// Path to module
        module_path: Option<PathBuf>,
    },
    /// Build the module
    Build {
        /// Path to module
        module_path: Option<PathBuf>,
    },
    /// Remove artifacts that builder generated
    Clean {
        /// Path to module
        module_path: Option<PathBuf>,
    },
    /// Build and seal a module
    Package {
        /// Remove the source code
        #[arg(short, long)]
        strip: bool,

        /// Path to module
        module_path: Option<PathBuf>,
    },
    /// Wrap an application as an executable file
    Wrap {
        /// Remove the source code
        #[arg(short, long)]
        strip: bool,

        /// Path to module
        module_path: Option<PathBuf>,
    },
    /// Display or disassemble the object file
    Dump {
        /// List all sections
        #[arg(short, long, group = "action")]
        list: bool,

        /// Display the content of the specified section
        #[arg(short, long, value_name = "section_name", group = "action")]
        section: Option<String>,

        /// Disassemble the specified function
        #[arg(short, long, value_name = "function_name", group = "action")]
        function: Option<String>,

        /// Display the content of the specified data
        #[arg(short, long, value_name = "data_name", group = "action")]
        data: Option<String>,

        /// Path to object file
        object_file: Option<PathBuf>,
    },
    /// Print compilation environment variables
    Env {
        /// Name of the environment variable
        name: Option<String>,
    },
    /// Debug an application
    Debug {
        /// Path to application
        application_path: Option<PathBuf>,
    },
    /// Launch a text editor
    Edit { file: PathBuf },
    /// Launch a Shell
    Shell {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>
    },
    /// Run a shell command
    Command {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>
    },
    /// Command utilities
    #[command(subcommand)]
    Me(MeCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum NewModuleType {
    /// Application
    App,

    /// Shared module
    Lib,

    /// Both Shared module and application
    Mix,

    /// Application with multiple executable units
    Bins,
}

#[derive(Debug, Subcommand)]
enum MeCommand {
    /// Generate man pages
    Manpage,

    /// Generate shell completion script
    Completion,
}

#[derive(Debug, Parser)]
#[command(name = "ancrt")]
#[command(version, about, long_about = None)]
// #[command(next_line_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args);
}
