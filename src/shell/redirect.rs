use std::{
    fs::File,
    io::Write,
    iter::Peekable,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{parser::Token, shell::error::ShellError};

pub(crate) fn redirect_builtin_output<'a, I>(
    redirect_symb: &str,
    builtin_out: &str,
    token_iter: &mut Peekable<I>,
) -> Result<(), ShellError>
where
    I: Iterator<Item = &'a Token>,
{
    let file_name = match token_iter.next() {
        Some(Token::Arg(file_name)) => file_name,
        Some(t) => return Err(ShellError::NoFileForRedirection(Some(t.to_owned())))?,
        None => return Err(ShellError::NoFileForRedirection(None))?,
    };
    let file_path = PathBuf::from(file_name);
    if let Some(parent_dir) = file_path.parent() {
        match std::fs::create_dir_all(parent_dir) {
            Ok(()) => {}
            Err(e) => return Err(ShellError::CouldNotCreateParentDir(file_path, e))?,
        }
    }

    let mut file_options = File::options();
    file_options.create(true).write(true);

    match redirect_symb {
        "2>" => {
            let _ = file_options
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            print!("{builtin_out}");
        }
        "2>>" => {
            file_options.append(true);
            let _ = file_options
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            print!("{builtin_out}");
        }
        ">>" | "1>>" => {
            // when writing to files linux adds a newline character at the end
            file_options.append(true);
            let mut file = file_options
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            file.write_all(builtin_out.as_bytes()).map_err(|e| {
                ShellError::WriteFileFailure(builtin_out.to_string(), file_path.clone(), e)
            })?;
        }
        ">" | "1>" => {
            // when writing to files linux adds a newline character at the end
            let mut file = file_options
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            file.write_all(builtin_out.as_bytes()).map_err(|e| {
                ShellError::WriteFileFailure(builtin_out.to_string(), file_path.clone(), e)
            })?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(crate) fn redirect_external<'a, I>(
    command: &mut Command,
    redirect_symb: &str,
    token_iter: &mut Peekable<I>,
) -> Result<(), ShellError>
where
    I: Iterator<Item = &'a Token>,
{
    let file_name = match token_iter.next() {
        Some(Token::Arg(file_name)) => file_name,
        Some(t) => return Err(ShellError::NoFileForRedirection(Some(t.to_owned())))?,
        None => return Err(ShellError::NoFileForRedirection(None))?,
    };
    let file_path = PathBuf::from(file_name);
    if let Some(parent_dir) = file_path.parent() {
        match std::fs::create_dir_all(parent_dir) {
            Ok(()) => {}
            Err(e) => return Err(ShellError::CouldNotCreateParentDir(file_path, e))?,
        }
    }

    let mut file_options = File::options();
    file_options.create(true).write(true);

    match redirect_symb {
        ">" | "1>" => {
            file_options.truncate(true);
            let file = file_options
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            command.stdout(Stdio::from(file));
        }
        "2>" => {
            file_options.truncate(true);
            let file = file_options
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            command.stderr(Stdio::from(file));
        }
        "2>>" => {
            let file = file_options
                .append(true)
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            command.stderr(Stdio::from(file));
        }
        ">>" | "1>>" => {
            let file = file_options
                .append(true)
                .open(&file_path)
                .map_err(|e| ShellError::FailedToOpenFile(file_path.clone(), e))?;
            command.stdout(Stdio::from(file));
        }
        _ => unreachable!("Unknown redirection operator"),
    }

    let mut child = command
        .spawn()
        .map_err(|e| ShellError::CommandSpawnFailure {
            name: file_name.into(),
            source: e,
        })?;
    child
        .wait()
        .map_err(|e| ShellError::CommandWaitFailure(child, e))?;
    Ok(())
}
