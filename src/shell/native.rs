pub fn run_command(command: &str) {
    use execute::Execute;

    let mut cmd = execute::shell(command);
    let output = cmd.execute_output().unwrap();
    println!("running command: {}", command);
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8(output.stdout).unwrap());
    println!("stderr: {}", String::from_utf8(output.stderr).unwrap());
    println!("========================");
}
