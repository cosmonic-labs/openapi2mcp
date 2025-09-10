pub fn run_command(command: &str, args: &[String]) -> anyhow::Result<()> {
    use execute::Execute;

    let mut cmd = execute::shell(command);
    cmd.args(args);

    let output = cmd.execute_output()?;
    println!("running command: {command}");
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8(output.stdout)?);
    println!("stderr: {}", String::from_utf8(output.stderr)?);
    println!("========================");
    Ok(())
}
