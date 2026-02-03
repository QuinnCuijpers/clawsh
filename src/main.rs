use clawsh::shell::Shell;

fn main() -> anyhow::Result<()> {
    let mut shell = Shell::setup()?;
    match shell.run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("ERROR: {e}");
        }
    }
    // TODO: handle errors for exiting
    shell.exit()
}
