// Compiles Arcana templates.
// Copyright (C) 2026  OC (oc@oh-clever.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use {
    arcana_core::{
        Context,
        Arcana,
    },
    std::{ io, path::PathBuf, },
};

fn help() -> ! {
	println!("{}", include_str!("../resources/help.txt"));
	std::process::exit(0)
}

fn get_short_version<'a>() -> &'a str {
    env!("CARGO_PKG_VERSION")
}

fn short_version() -> ! {
    println!("{}", get_short_version());
    std::process::exit(0)
}

fn set<P: AsRef<std::path::Path>>(ctx: &mut Context, pwd: P, dkv: String) -> bool {
    if dkv.is_empty() {
        return false;
    }

    let mut chars = dkv.chars();
    let dlim = chars.next().unwrap();
    let rest = chars.collect::<String>();
    if rest.is_empty() {
        return false;
    }

    let mut split = rest.split(dlim);

    let key = match split.next() {
        Some(key) => key,
        None => return false,
    };

    let val = match split.next() {
        Some(key) => key,
        None => return false,
    };

    if split.next().is_some() {
        return false;
    }

    ctx.add_variable(key, pwd, val);

    true
}

fn version() -> ! {
    println!("arcc: v{}", get_short_version());
	std::process::exit(0)
}

fn main() {
    let pwd = std::env::current_dir().unwrap();

    let mut path: Option<PathBuf> = None;
    let mut read_stdin = false;

    let mut ctx = Context::default();

    let mut args = std::env::args();
    args.next(); // burn program name

    while let Some(full_arg) = args.next() {
        if let Some(long_arg) = full_arg.strip_prefix("--") {
            match long_arg {
                "help" => help(),
                "set" => {
                    let arg = match args.next() {
                        Some(arg) => arg,
                        None => {
                            eprintln!("arcc: --set requires a value");
                            std::process::exit(1);
                        },
                    };

                    if !set(&mut ctx, &pwd, arg) {
                        eprintln!("arcc: invalid <DKV> passed to --set");
                        std::process::exit(1);
                    }
                },
                "version" => version(),
                long_arg => {
                    eprintln!("arcc: unknown argument '--{long_arg}'");
                    std::process::exit(1);
                },
            }
        }
        else if full_arg.starts_with('-') && full_arg.len() > 1 {
            let mut short_args = full_arg[1..].chars();
            match short_args.next() {
                Some('h') => help(),
                Some('s') => {
                    if short_args.next().is_some() {
                        eprintln!("arcc: -s requires a value");
                        std::process::exit(1);
                    }

                    let arg = match args.next() {
                        Some(arg) => arg,
                        None => {
                            eprintln!("arcc: -s requires a value");
                            std::process::exit(1);
                        },
                    };

                    if !set(&mut ctx, &pwd, arg) {
                        eprintln!("arcc: invalid <DKV> passed to -s");
                        std::process::exit(1);
                    }
                },
                Some('v') => short_version(),
                Some(short_arg) => {
                    eprintln!("arcc: unknown arguemnt '-{short_arg}'");
                    std::process::exit(1);
                },
                _ => panic!("HOW WAS THE ARG NONE!?"),
            }
        }
        // just a hyphen, signals read from stdin
        else if full_arg.starts_with('-') {
            read_stdin = true;

            break
        }
        else {
            if path.is_some() {
                eprintln!("arcc: cannot include more than one path");
                std::process::exit(1);
            }

            path = Some(PathBuf::from(full_arg));

            break
        }
    }

    if path.is_none() && !read_stdin {
        eprintln!("arcc: path must be defined");
        std::process::exit(1);
    }
    else if let Some(arg) = args.next() {
        let mut trailing = Vec::new();
        trailing.push(arg);

        for arg in args {
            trailing.push(arg);
        }

        eprintln!("arcc: trailing arguments: {}", trailing.join(" "));
        std::process::exit(1);
    }
    else if read_stdin {
        if let Err(e) = Arcana::compile_to_stdout_with_ctx(io::stdin(), ctx) {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
    else if let Err(e) = Arcana::compile_file_to_stdout_with_ctx(path.unwrap(), ctx) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
