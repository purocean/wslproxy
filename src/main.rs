use std::env;
use std::process::{Command, Stdio};
use std::path::Path;

fn translate_path_to_unix(arg: String) -> String {
    if let Some(index) = arg.find(":\\") {
        if index != 1 {
            // Not a path
            return arg;
        }
        let mut path_chars = arg.chars();
        if let Some(drive) = path_chars.next() {
            let mut wsl_path = String::from("/mnt/");
            wsl_path.push_str(&drive.to_lowercase().collect::<String>());
            path_chars.next();
            wsl_path.push_str(&path_chars.map(|c|
                    match c {
                        '\\' => '/',
                        _ => c,
                    }
                ).collect::<String>());
            return wsl_path;
        }
    } else {
        // maybe a relative path
        if let Some(backslash_index) = arg.find("\\") {
            // \somepach not a relative path
            if backslash_index > 1 {
                if Path::new(&arg).exists() {
                    return arg.replace("\\", "/").to_string();
                }
            }
        }
    }
    arg
}

fn translate_path_to_win(line: &str) -> String {
    if let Some(index) = line.find("/mnt/") {
        if index != 0 {
            // Path somewhere in the middle, don't change
            return String::from(line);
        }
        let mut path_chars = line.chars();
        if let Some(drive) = path_chars.nth(5) {
            if let Some(slash) = path_chars.next() {
                if slash != '/' {
                    // not a windows mount
                    return String::from(line);
                }
                let mut win_path = String::from(
                    drive.to_lowercase().collect::<String>());
                win_path.push_str(":\\");
                win_path.push_str(&path_chars.collect::<String>());
                return win_path.replace("/", "\\").to_string();
            }
        }
    }
    String::from(line)
}

fn shell_escape(arg: String) -> String {
    // ToDo: This really only handles arguments with spaces.
    // More complete shell escaping is required for the general case.
    if arg.contains(" ") {
        return vec![
            String::from("\""),
            arg,
            String::from("\"")].join("");
    }
    arg
}

fn get_program_name() -> String {
    let current_exe_path = env::current_exe().unwrap();
    let current_file_name = current_exe_path.file_stem().unwrap().to_str().unwrap();
    return current_file_name.to_string();
}

fn main() {
    let mut git_args: Vec<String> = vec![get_program_name()];
    git_args.extend(env::args().skip(1)
        .map(translate_path_to_unix)
        .map(shell_escape));
    let git_cmd = git_args.join(" ");
    let git_proc = Command::new("bash")
        .arg("-i")
        .arg("-c")
        .arg(&git_cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to execute command '{}'", &git_cmd));
    let output = git_proc
        .wait_with_output()
        .expect(&format!("Failed to wait for git call '{}'", &git_cmd));
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines().map(translate_path_to_win) {
        println!("{}", line);
    }
}
