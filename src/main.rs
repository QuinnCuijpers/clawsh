use anyhow::{Context, anyhow};
use faccess::PathExt;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env::{current_dir, set_current_dir}, path::{Path, PathBuf}, process::Command, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Builtin {
    Echo,
    Exit,
    Tipe,
    Pwd,
    Cd,
}

impl FromStr for Builtin {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtin::Echo),
            "exit" => Ok(Builtin::Exit),
            "type" => Ok(Builtin::Tipe),
            "pwd" => Ok(Builtin::Pwd),
            "cd" => Ok(Builtin::Cd),
            _ => Err(anyhow!(format!("unknown builtin {s}"))),
        }
    }
}

fn main() -> anyhow::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush().context("flushing stdout")?;
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf).context("reading stdin")?;
        let mut input = buf.trim_end();
        let mut command_list = vec![];

        while let Some((s1, mut s2)) = input.split_once(" ") {
            if !s1.starts_with("\"") {
                command_list.push(s1.to_string());
                input = s2;
                continue;
            }
            if let Some(idx) = s2.find("\"") {
                let mut combined = s1.to_string();
                combined.push_str(" ");
                combined.push_str(&s2[..idx + 1]);
                s2 = &s2[idx + 1..];
                command_list.push(combined);
                input = s2;
            } else {
                return Err(anyhow::anyhow!("unclosed qoutes for {s1}"));
            }
        }
        // all input that can no longer be split on space is still added to the command list
        if !input.is_empty() {
            command_list.push(input.to_string());
        }

        if let Ok(command) = Builtin::from_str(&command_list[0]) {
            match command {
                Builtin::Echo => invoke_echo(&command_list[1..]),
                Builtin::Exit => break,
                Builtin::Tipe => invoke_type(&command_list[1..]),
                Builtin::Pwd => invoke_pwd(&command_list[1..]),
                Builtin::Cd => invoke_cd(&command_list[1..]),
            }
        } else {
            // TODO: add support for executing files
            let Some(env_path) = std::env::var_os("PATH") else {
                panic!("PATH env var not set");
            };
            let exec = &command_list[0];
            if let Some(_) = find_exec_file(exec, env_path) {
                let mut output = Command::new(exec).args(&command_list[1..]).spawn()?;
                output.wait()?;
                continue;
            }
            println!("{input}: command not found")
        }
    }
    anyhow::Ok(())
}

fn invoke_echo(cmd_list: &[String]) {
    let out = cmd_list.join(" ");
    println!("{out}");
}

fn invoke_type(cmd_list: &[String]) {
    for cmd in cmd_list {
        if let Ok(_) = Builtin::from_str(cmd) {
            println!("{cmd} is a shell builtin");
            return;
        }
        // go through every directory and check if a file with the name exist that has exec permissions
        let Some(env_path) = std::env::var_os("PATH") else {
            panic!("PATH env var not set");
        };
        if let Some(file_path) = find_exec_file(&cmd, env_path) {
            println!("{cmd} is {}", file_path.display());
        } else {
            println!("{cmd}: not found");
        }
    }
}

fn find_exec_file(cmd: &str, env_path: std::ffi::OsString) -> Option<PathBuf> {
    for path in std::env::split_paths(&env_path) {
        if let Ok(exists) = path.try_exists() {
            if !exists {
                continue;
            }
            for dir in path.read_dir().expect("dir should exist") {
                if let Ok(dir) = dir {
                    let file_name = dir.file_name();
                    let file_path = dir.path();
                    if file_name == cmd && file_path.executable() {
                        return Some(file_path);
                    }
                }
            }
        }
    }
    None
}

fn invoke_pwd(_cmd_list: &[String]) {
    if let Ok(curr) = current_dir() {
        println!("{}", curr.display());
    }
}

fn invoke_cd(cmd_list: &[String]) {
    assert!(cmd_list.len() == 1);

    let path = Path::new(&cmd_list[0]);
    if path.exists() {
        match set_current_dir(path) {
            Ok(()) => return,
            Err(e) => panic!("could not cd to {} due to {e}", path.display()),
        }
    }
    else {
        println!("cd: {}: No such file or directory", path.display());
    }
}
