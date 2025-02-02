use std::collections::HashMap;

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
