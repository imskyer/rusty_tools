use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

use std::io::BufRead;
use std::sync::mpsc;

use uuid::*;

#[derive(Debug)]
pub struct CompileOutput {
    pub output: std::path::PathBuf,
    pub errors: i32,
}

pub struct ResidentCompiler {
    writer: Option<std::io::BufWriter<std::process::ChildStdin>>,
    incremental_output: Option<mpsc::Receiver<CompileOutput>>,
}

impl ResidentCompiler {
    pub fn new() -> ResidentCompiler {
        ResidentCompiler {
            writer: None,
            incremental_output: None,
        }
    }

    /// Start the frontend server.
    ///
    /// All of these paths are hardcoded here, but should be known ahead of time anyway.
    pub fn start(&mut self) {
        let (sender, receiver) = mpsc::channel();
        let frontend_server = Command::new("/Users/jonahwilliams/Documents/flutter/bin/cache/dart-sdk/bin/dart")
            .arg("/Users/jonahwilliams/Documents/flutter/bin/cache/artifacts/engine/darwin-x64/frontend_server.dart.snapshot")
            .arg("--sdk-root=/Users/jonahwilliams/Documents/flutter/bin/cache/artifacts/engine/common/flutter_patched_sdk/")
            .arg("--strong")
            .arg("--incremental")
            .arg("--target=flutter")
            .arg("--packages=/Users/jonahwilliams/Documents/flutter/examples/flutter_gallery/.packages")
            .arg("--output-dill=build/app.dill")
            .arg("--filesystem-scheme=org-dartlang-root")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn frontend server");

        let reader = frontend_server.stdout.unwrap();
        let mut writer = io::BufWriter::new(frontend_server.stdin.unwrap());
        let mut output_handler = CompilerOutputHandler {
            boundary_key: String::new(),
            compilation_request: sender,
        };
        std::thread::spawn(move || {
            let reader = io::BufReader::new(reader);
            for line in reader.lines() {
                if let Ok(line) = line {
                    output_handler.on_line(&line);
                }
            }
        });
        write!(writer, "compile {}\n", "package:flutter_gallery/main.dart").unwrap();
        writer.flush().unwrap();
        receiver.recv().unwrap();
        self.writer = Some(writer);
        self.incremental_output = Some(receiver);
    }

    pub fn recompile<'b, T>(&mut self, main: &std::path::Path, invalidated: T) -> CompileOutput
    where
        T: Iterator<Item = &'b std::path::Path>,
    {
        let writer = self.writer.as_mut().unwrap();
        let input_key = Uuid::new_v4();
        let main_package = ResidentCompiler::hacky_mapper(main); // hard-coded.
        write!(writer, "recompile {} {}\n", main_package, input_key).unwrap();
        for file in invalidated {
            write!(writer, "{}\n", ResidentCompiler::hacky_mapper(file)).unwrap();
        }
        write!(writer, "{}\n", input_key).unwrap();
        writer.flush().unwrap();
        self.incremental_output.as_mut().unwrap().recv().unwrap()
    }

    pub fn accept(&mut self) {
        let writer = self.writer.as_mut().unwrap();
        writer.write(b"accept\n").unwrap();
    }

    pub fn reject(&mut self) {
        let writer = self.writer.as_mut().unwrap();
        writer.write(b"reject\n").unwrap();
    }

    // pub fn reset(&mut self) {
    //     let writer = self.writer.as_mut().unwrap();
    //     writer.write(b"reset\n").unwrap();
    //     writer.flush().unwrap();
    // }

    fn hacky_mapper<'b>(path: &'b std::path::Path) -> String {
        let segments = path
            .to_str()
            .unwrap()
            .split("/")
            .skip_while(|segment| segment != &"lib")
            .skip(1);
        let mut result = String::from("package:flutter_gallery");
        for segment in segments {
            result.push('/');
            result.push_str(segment);
        }
        result
    }
}

struct CompilerOutputHandler {
    boundary_key: String,
    compilation_request: mpsc::Sender<CompileOutput>,
}

impl CompilerOutputHandler {
    fn on_line(&mut self, line: &str) {
        if self.boundary_key.is_empty() && line.starts_with("result ") {
            self.boundary_key.push_str(line.split_at("result ".len()).1);
            return;
        } else if line.starts_with(&self.boundary_key) {
            if line.len() <= self.boundary_key.len() {
                return; // ERROR
            }
            let mut sections = line.split(" ").skip(1);
            let output = CompileOutput {
                output: std::path::Path::new(sections.next().unwrap()).to_path_buf(),
                errors: sections.next().unwrap().parse::<i32>().unwrap(),
            };
            self.boundary_key.clear();
            self.compilation_request.send(output).unwrap();
        }
    }
}
