use std::{ffi::OsStr, fs::File, io::Write, iter::Peekable, path::PathBuf, process::Child, str::FromStr as _};

use rustyline::history::FileHistory;

use crate::{commands::{Builtin, invoke_builtin}, parser::{Token, split_words}, shell::exec::handle_external_exec};


pub(crate) fn handle_builtin<'a, S, I, J>(
    builtin: Builtin,
    args: J,
    token_iter: &mut Peekable<I>,
    prev_command_output: Option<String>,
    _prev_command: Option<&mut Child>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
    J: Iterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut all_args: Vec<String> = args
        .map(|s| s.as_ref().to_str().unwrap().to_string())
        .collect();

    if let Some(out) = prev_command_output {
        let extra_args = split_words(&out);
        all_args.extend(extra_args);
    }

    // if let Some(child) = prev_command
    // && let Some(stdout) = child.stdout.as_mut()
    // {
    //     // builtins do not read stdin
    // }

    let Some(builtin_out) = invoke_builtin(builtin, all_args.iter(), history) else {
        // early return for cd
        return Ok(());
    };

    match token_iter.next() {
        None => print!("{builtin_out}"),
        Some(Token::Redirect(c)) => {
            let Some(Token::Arg(file_name)) = token_iter.next() else {
                anyhow::bail! {"expected file name after redirection"};
            };
            let file_path = PathBuf::from(file_name);
            if let Some(parent_dir) = file_path.parent()
                && std::fs::create_dir_all(parent_dir).is_err()
            {
                anyhow::bail!("Failed to create dirs required for {}", file_path.display());
            }

            let mut file_options = File::options();
            file_options.create(true).write(true);

            match c.as_str() {
                "2>" => {
                    let _ = file_options
                        .open(file_path)
                        .expect("couldnt open file for redirecting stderr");
                    print!("{builtin_out}");
                }
                "2>>" => {
                    file_options.append(true);
                    let _ = file_options
                        .open(file_path)
                        .expect("couldnt open file for appending stderr");
                    print!("{builtin_out}");
                }
                ">>" | "1>>" => {
                    // when writing to files linux adds a newline character at the end
                    file_options.append(true);
                    let mut file = file_options
                        .open(file_path)
                        .expect("couldnt open file for stdout appending");
                    let _ = file.write_all(builtin_out.as_bytes());
                }
                ">" | "1>" => {
                    // when writing to files linux adds a newline character at the end
                    let mut file = file_options
                        .open(file_path)
                        .expect("couldnt open file for stdout redirection");
                    let _ = file.write_all(builtin_out.as_bytes());
                }
                _ => unreachable!(),
            }
        }
        Some(Token::Pipe) => {
            let Some(Token::Command(cmd)) = token_iter.next() else {
                anyhow::bail!("Piped into nothing");
            };

            let mut next_args = vec![];
            while let Some(Token::Arg(s)) = token_iter.peek() {
                next_args.push(s.clone());
                token_iter.next();
            }

            // create pipeline recursively
            if let Ok(cmd) = Builtin::from_str(cmd) {
                handle_builtin(
                    cmd,
                    next_args.iter(),
                    token_iter,
                    Some(builtin_out),
                    None,
                    history,
                )?;
            } else {
                handle_external_exec(
                    cmd,
                    next_args.iter(),
                    token_iter,
                    Some(builtin_out),
                    None,
                    history,
                )?;
            }
        }
        Some(_t) => unreachable!(),
    }
    Ok(())
}