// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// a script application may consist of a single script file, or several script files.
// in either case, these script files will be compiled into a single module image file
// named 'main.ancbc', and this file will be copied to the 'application cache' directory.
//
// the dependent modules of application are copied to this cache directory also, but
// the shared modules (such as the standard library) are located at the runtime directory, and
// they will not be copied to this directory.
//
// the structure of application cache directory
//
// app cache dir
//   |-- cache.index
//   |   the cache infomations, such as the file type, version (dependencies only),
//   |   and the last modified time, content hash (only the application script files), and the module list
//   |
//   |-- main_module.ancbc
//   |-- dependency_module1.ancbc
//   |-- ...
//   |-- dependency_moduleN.ancbc
//
// all module files and the cache dir itself are named with:
// - `dev_num_major + '_' + dev_num_minor + '_' + inode_num`, or
// - the sha256 of the full path.
//
// note that apart from the script files, a (multiple scripts) application may also contains resource files
// and (dynamically linked) shared libraries.
// these files will stay in their original location and will not be copied to the cache directory.
//
// app source file dir
//   |-- module.anon
//   |   the application description file, similar to the Nodejs's package.json
//   |   and the Rust's Cargo.toml
//   |
//   |-- main.ancs
//   |   the main module script file, the first line would be '#!/bin/ancs -d' (ref: https://en.wikipedia.org/wiki/Shebang_(Unix))
//   |
//   |-- sub-module.ancs
//   |-- sub-dir
//   |     |-- sub-module.ancs
//   |     |-- ...
//   |
//   |-- resources
//   |     |-- images_etc
//   |     |-- ...
//   |
//   |-- lib
//   |     |-- shared-library.so -> shared-library.so.1.0.0
//   |     |-- ...

// to launch a script application which is a single file:
// - `$ ancs /path/to/single-file-app.ancs`
// - `$ /path/to/single-file-app.ancs`
// - `$ single-file-app.ancs` (which path is in the environment variable PATH)
// - `$ [/usr/bin/]single-file-app` (a symbolic link to '/path/to/single-file-app.ancs')
//
// to lanuch a script application which is consist of multiple script files:
// - `$ ancs /path/to/app-source-dir`
// - `$ ancs /path/to/app-source-dir/main.ancs`
// - `$ /path/to/app-source-dir/main.ancs`
// - `$ [/usr/bin/]app-name` (a symblic link to '/path/to/app-source-dir/main.ancs')

use ancasm_assembler::{
    assembler::assemble_merged_module_node, imagegenerator::generate_module_image_binary,
    linker::link, preprocessor::merge_and_canonicalize_submodule_nodes,
};
use ancasm_parser::{
    lexer::{filter, lex},
    parser::parse,
    peekable_iterator::PeekableIterator,
};
use ancvm_processor::{
    in_memory_program_resource::InMemoryProgramResource, process::start_program_in_multithread,
};

const EXIT_FAILURE: i32 = 1;

fn main() {
    let module_binary = helper_generate_module_image_binary_from_str(
        r#"
        (module $app
            (runtime_version "1.0")
            (function $test (results i64)
                (code
                    (i64.imm 0)
                )
            )
        )
        "#,
    );

    let program_resource0 = InMemoryProgramResource::new(vec![module_binary]);
    let result0 = start_program_in_multithread(program_resource0, vec![]);

    match result0 {
        Ok(exit_code) => std::process::exit(exit_code as i32),
        Err(e) => {
            eprintln!("Application error: {}", e);
            std::process::exit(EXIT_FAILURE);
        }
    }
}

pub fn helper_generate_module_image_binary_from_str(source: &str) -> Vec<u8> {
    let mut chars = source.chars();
    let mut char_iter = PeekableIterator::new(&mut chars, 3);
    let all_tokens = lex(&mut char_iter).unwrap();
    let effective_tokens = filter(&all_tokens);
    let mut token_iter = effective_tokens.into_iter();
    let mut peekable_token_iter = PeekableIterator::new(&mut token_iter, 2);

    let module_node = parse(&mut peekable_token_iter, None).unwrap();
    let merged_module_node =
        merge_and_canonicalize_submodule_nodes(&[module_node], None, None).unwrap();

    let (module_entry, _) = assemble_merged_module_node(&merged_module_node).unwrap();
    let module_entries = vec![&module_entry];

    // let program_settings = ProgramSettings::default();
    let index_entry = link(&module_entries, 0).unwrap();
    generate_module_image_binary(&module_entry, Some(&index_entry)).unwrap()
}
