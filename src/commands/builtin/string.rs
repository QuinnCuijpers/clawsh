use crate::commands::{Builtin, find_exec_file};
use std::str::FromStr;

pub(crate) fn invoke_echo(cmd_list: &[String]) -> String {
    let mut s = cmd_list.join(" ");
    s.push('\n');
    s
}

pub(crate) fn invoke_type(cmd_list: &[String]) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    for (i, cmd) in cmd_list.iter().enumerate() {
        let cmd_str = if i != 0 {
            format!("\n{cmd}")
        } else {
            cmd.clone()
        };
        let cmd_str = cmd_str.as_str();
        if Builtin::from_str(cmd).is_ok() {
            let _ = write!(buf, "{cmd_str} is a shell builtin");
        } else {
            // go through every directory and check if a file with the name exist that has exec permissions
            match find_exec_file(cmd) {
                Ok(Some(file_path)) => {
                    let _ = write!(buf, "{cmd_str} is {}", file_path.display());
                }
                Ok(None) => {
                    let _ = write!(buf, "{cmd_str}: not found");
                }
                Err(_e) => {}
            }
        }
    }
    buf.push('\n');
    buf
}
