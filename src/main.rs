// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{io::Write, path::PathBuf};

use anc_isa::ModuleDependencyType;
use anc_runtime::{
    builder::build_application_by_dependency_tree, entry::RuntimeProperty, RuntimeError,
};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, generate_to, Shell};

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

        /// Include unit tests
        #[arg(short, long)]
        tests: bool,
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
        command_line: Vec<String>,
    },
    /// Run a shell command
    Command {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command_line: Vec<String>,
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

    /// Single-file application
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ShellType {
    Bash,
    Zsh,
    Fish,
    NuShell,
    PowerShell,
}

#[derive(Debug, Subcommand)]
enum MeCommand {
    /// Generate man pages
    Manpage {
        /*
         * The Linux man page files are usually placed in:
         * - `/usr/share/man/{man1,man2,man3,...}`
         * - `/usr/local/share/man/`
         * - `~/.local/share/man`
         *
         * where
         *
         * `man1` for "Executable programs or shell commands",
         * `man2` for "System calls (functions provided by the kernel)",
         * `man3` for "Library calls (functions within program libraries)".
         *
         * check out env variable $MANPATH and `/etc/man_db.conf` for details.
         */
        /// Directory to save the Linux man page file
        out_dir: Option<PathBuf>,
    },

    /// Generate shell completion script
    Completion {
        /*
         * the Bash completion scripts are usually placed in:
         *
         * - `/usr/share/bash-completion/completions/`
         * - `/etc/bash_completion.d/`
         * - `~/.local/share/bash-completion/completions/`
         *
         * and they are loaded by scripts:
         *
         * - `/etc/bash_completion`
         * - `/etc/profile.d/bash_completion.sh`
         * - `/usr/share/bash-completion/bash_completion`
         *
         * check out `/etc/profile`, `/etc/bashrc` and `/etc/bash.bashrc` for details.
         */
        /// Shell type
        #[arg(short, long)]
        #[arg(value_enum)]
        #[arg(default_value = "bash")]
        shell: ShellType,

        /// Directory to save the script file
        out_dir: Option<PathBuf>,
    },
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
    if let Err(err) = process_cmd() {
        let mut stderr = std::io::stderr();
        let message = match err {
            RuntimeError::Message(m) => m,
        };
        writeln!(&mut stderr).unwrap();
        writeln!(&mut stderr, "{}", message).unwrap();
        std::process::exit(1);
    }
}

fn process_cmd() -> Result<(), RuntimeError> {
    let mut stdout = std::io::stdout();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            application_path,
            args,
        } => todo!(),
        Commands::New {
            type_,
            module_name,
            location,
        } => todo!(),
        Commands::Test {
            path_name_prefix,
            module_path,
        } => todo!(),
        Commands::Build { module_path, tests } => {
            let path = if let Some(path) = module_path {
                path
            } else {
                PathBuf::from(".")
            };

            let full_path = path.canonicalize().unwrap();
            let runtime_property = RuntimeProperty::from_runtime_exec_file()?;

            if full_path.is_file() {
                // build_application_by_single_file(&full_path, &runtime_property)?;
                Err(RuntimeError::Message(
                    "Single-file application do not need to be built, use the `anc run` command to run it directly.".to_owned()))
            } else {
                build_application_by_dependency_tree(
                    &full_path,
                    ModuleDependencyType::Local,
                    &runtime_property,
                    tests,
                    &mut stdout,
                )?;
                Ok(())
            }
        }
        Commands::Clean { module_path } => todo!(),
        Commands::Package { strip, module_path } => todo!(),
        Commands::Wrap { strip, module_path } => todo!(),
        Commands::Dump {
            list,
            section,
            function,
            data,
            object_file,
        } => todo!(),
        Commands::Env { name } => todo!(),
        Commands::Debug { application_path } => todo!(),
        Commands::Edit { file } => todo!(),
        Commands::Shell { command_line } => todo!(),
        Commands::Command { command_line } => todo!(),
        Commands::Me(me_command) => match me_command {
            MeCommand::Manpage { out_dir } => {
                let cmd = Cli::command();
                let man = clap_mangen::Man::new(cmd);
                if let Some(dir) = out_dir {
                    man.generate_to(dir).unwrap();
                } else {
                    man.render(&mut stdout).unwrap();
                }
                Ok(())
            }
            MeCommand::Completion { shell, out_dir } => {
                let mut cmd = Cli::command();
                if matches!(shell, ShellType::NuShell) {
                    if let Some(dir) = out_dir {
                        generate_to(clap_complete_nushell::Nushell, &mut cmd, "ancrt", dir)
                            .unwrap();
                    } else {
                        generate(
                            clap_complete_nushell::Nushell,
                            &mut cmd,
                            "ancrt",
                            &mut stdout,
                        );
                    }
                } else {
                    let generator = match shell {
                        ShellType::Bash => Shell::Bash,
                        ShellType::Zsh => Shell::Zsh,
                        ShellType::Fish => Shell::Fish,
                        ShellType::NuShell => unreachable!(),
                        ShellType::PowerShell => Shell::PowerShell,
                    };
                    if let Some(dir) = out_dir {
                        generate_to(generator, &mut cmd, "ancrt", dir).unwrap();
                    } else {
                        generate(generator, &mut cmd, "ancrt", &mut stdout);
                    }
                }
                Ok(())
            }
        },
    }
}
