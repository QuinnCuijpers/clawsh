use std::{ffi::OsStr, fs::File, io::Write, iter::Peekable, path::PathBuf, process::{Child, Command, Stdio}, str::FromStr as _};

use rustyline::history::FileHistory;

use crate::{commands::Builtin, parser::Token, shell::builtin_exec::handle_builtin};


pub(crate) fn handle_external_exec<'a, S, I, J>(
    cmd_str: &str,
    args: J,
    token_iter: &mut Peekable<I>,
    prev_command_output: Option<String>,
    prev_command: Option<&mut Child>,
    history: &mut FileHistory,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a Token>,
    J: Iterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd_str);

    command.args(args);

    match token_iter.next() {
        // no more tokens
        None => {
            if prev_command_output.is_some() {
                command.stdin(Stdio::piped());
            } else if let Some(prev) = prev_command
                && prev.stdout.is_some()
            {
                command.stdin(prev.stdout.take().unwrap());
            }

            let mut child = command.spawn()?;

            if let Some(prev) = prev_command_output {
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(prev.as_bytes())?;
            }

            child.wait()?;
        }
        Some(Token::Redirect(c)) => {
            let Some(Token::Arg(file_name)) = token_iter.next() else {
                anyhow::bail!("expected file name after redirection");
            };

            let file_path = PathBuf::from(file_name);
            if let Some(parent_dir) = file_path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }

            let mut file_options = File::options();
            file_options.create(true).write(true);

            match c.as_str() {
                ">" | "1>" => {
                    file_options.truncate(true);
                    let file = file_options.open(file_path)?;
                    command.stdout(Stdio::from(file));
                }
                "2>" => {
                    file_options.truncate(true);
                    let file = file_options.open(file_path)?;
                    command.stderr(Stdio::from(file));
                }
                "2>>" => {
                    let file = file_options.append(true).open(file_path)?;
                    command.stderr(Stdio::from(file));
                }
                ">>" | "1>>" => {
                    let file = file_options.append(true).open(file_path)?;
                    command.stdout(Stdio::from(file));
                }
                _ => unreachable!("Unknown redirection operator"),
            }

            let mut child = command.spawn()?;
            child.wait()?;
        }
        Some(Token::Pipe) => {
            command.stdout(Stdio::piped());

            if let Some(prev) = prev_command
                && let Some(stdout) = prev.stdout.take()
            {
                command.stdin(stdout);
            }

            if prev_command_output.is_some() {
                command.stdin(Stdio::piped());
            }

            let mut child = command.spawn()?;

            if let Some(prev) = prev_command_output {
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(prev.as_bytes())?;
                drop(stdin);
            }

            let Some(Token::Command(cmd)) = token_iter.next() else {
                anyhow::bail!("Piped into nothing");
            };

            let mut next_args = vec![];
            while let Some(Token::Arg(s)) = token_iter.peek() {
                next_args.push(s);
                token_iter.next();
            }

            // create pipeline recursively
            if let Ok(cmd) = Builtin::from_str(cmd) {
                handle_builtin(
                    cmd,
                    next_args.iter(),
                    token_iter,
                    None,
                    Some(&mut child),
                    history,
                )?;
            } else {
                handle_external_exec(
                    cmd,
                    next_args.iter(),
                    token_iter,
                    None,
                    Some(&mut child),
                    history,
                )?;
            }

            child.wait()?;
        }
        Some(t) => unreachable!("found unhandled token: {:?}", t),
    }
    Ok(())
}