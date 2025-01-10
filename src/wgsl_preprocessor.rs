use std::{
    env, fs,
    io::Error,
    path::{Path, PathBuf},
};

use log::debug;

fn get_code(shader_file: &str) -> Result<String, Error> {
    let current_dir = env::current_dir()?;

    let shader_path = current_dir.join("src").join(shader_file);
    debug!("{:?}", shader_path);

    fs::read_to_string(shader_path)
}

pub fn process(shader_file: &str) -> String {
    match get_code(shader_file) {
        Ok(shader_code) => {
            let path = Path::new(shader_file);
            let directory = path.parent().unwrap();
            shader_code
                .lines()
                .map(|line| {
                    let line_tr = line.trim();
                    if line_tr.starts_with("//#include") {
                        // COMMAND
                        let filename = line[10..].trim();

                        let filename_with_dir = directory.join(filename);

                        match filename_with_dir.to_str() {
                            None => {
                                eprintln!("Error reading included file: {}", filename);
                                String::new()
                            }

                            Some(filename) => process(filename),
                        }
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        }
        Err(e) => {
            eprintln!("Error reading included file: {}", e);
            String::new()
        }
    }
    // return shader_code;
}
