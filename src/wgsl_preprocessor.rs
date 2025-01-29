use std::collections::HashMap;
use std::{
    env, fs,
    io::Error,
    path::{Path, PathBuf},
};

use log::debug;

pub struct WgslPreprocessor {
    shader_code: HashMap<String, String>,
}

impl WgslPreprocessor {
    fn process(shader_code: &HashMap<&'static str, &'static str>, file_name: &String) -> String {
        match shader_code.get(file_name.as_str()) {
            Some(code) => {
                // let path = Path::new(shader_file);
                // let directory = path.parent().unwrap();
                code.lines()
                    .map(|line| {
                        let line_tr = line.trim();
                        if line_tr.starts_with("//#include") {
                            // COMMAND
                            let filename = line[10..].trim().to_string();

                            Self::process(&shader_code, &filename)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            None => {
                eprintln!("Error reading included file in {}", file_name);
                String::new()
            }
        }
    }
    pub fn new(shader_code: HashMap<&'static str, &'static str>) -> Self {
        let new_shader_code = shader_code
            .keys()
            .fold(HashMap::new(), |mut acc, file_name| {
                let code = Self::process(&shader_code, &file_name.to_string());
                acc.insert(file_name.to_string(), code);
                acc
            });

        Self {
            shader_code: new_shader_code,
        }
    }

    pub fn get_code(&self, file_name: String) -> String {
        assert!(self.shader_code.contains_key(&file_name));
        self.shader_code.get(&file_name).unwrap().clone()
    }
}

// fn get_code(shader_file: &str) -> Result<String, Error> {
//     let current_dir = env::current_dir()?;

//     let shader_path = current_dir.join("src/shaders/").join(shader_file);
//     debug!("{:?}", shader_path);

//     fs::read_to_string(shader_path)
// }

// pub fn process(shader_file: &str) -> String {
//     match get_code(shader_file) {
//         Ok(shader_code) => {
//             let path = Path::new(shader_file);
//             let directory = path.parent().unwrap();
//             shader_code
//                 .lines()
//                 .map(|line| {
//                     let line_tr = line.trim();
//                     if line_tr.starts_with("//#include") {
//                         // COMMAND
//                         let filename = line[10..].trim();

//                         let filename_with_dir = directory.join(filename);

//                         match filename_with_dir.to_str() {
//                             None => {
//                                 eprintln!("Error reading included file: {}", filename);
//                                 String::new()
//                             }

//                             Some(filename) => process(filename),
//                         }
//                     } else {
//                         line.to_string()
//                     }
//                 })
//                 .collect::<Vec<String>>()
//                 .join("\n")
//         }
//         Err(e) => {
//             eprintln!("Error reading included file: {}", e);
//             String::new()
//         }
//     }
//     // return shader_code;
// }
