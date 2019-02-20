use getopts::Options;
use std::env;
use std::time::SystemTime;

use super::compile::*;
use super::devfs::*;
use super::vmservice::*;

const DEBUG_PORT_ARG: &'static str = "debug-port";

pub fn run() -> Result<(), ()> {
    // Configure and collect command line arguments.
    // Only debug-port is supported to make the example smaller.
    let mut opts = Options::new();
    opts.optopt(
        "p",
        DEBUG_PORT_ARG,
        "the observatory port on the device.",
        "12345",
    );
    let args: Vec<String> = env::args().collect();
    let matches = opts.parse(&args[2..]).map_err(handle_error)?;
    let port: u16 = match matches.opt_str(DEBUG_PORT_ARG) {
        Some(port) => port.parse::<u16>().unwrap(),
        None => panic!("--debug-port must be provided"),
    };
    // Connect to the vm service.
    let mut methods = VMServiceMethods::new();
    let mut vm_service = VMService::connect(&format!("ws://127.0.0.1:{}/ws", port))?;
    let mut devfs = DevFS::init(std::path::Path::new("lib/"))?;
    vm_service
        .send_notification(&methods.register_service("reloadSources", "flutter tools"))
        .map_err(handle_error)?;

    // Create the devfs, or destroy and then create it.
    let devfs_response: DevFSCreatedResponse =
        match vm_service.call_method(&methods.create_devfs("flutter_gallery")) {
            Ok(res) => res,
            Err(_) => {
                vm_service
                    .send_notification(&methods.delete_devfs("flutter_gallery"))
                    .unwrap();
                vm_service
                    .call_method(&methods.create_devfs("flutter_gallery"))
                    .unwrap()
            }
        };

    // Setup resident compiler.
    let mut resident_compiler = ResidentCompiler::new();
    resident_compiler.start();
    resident_compiler.accept();
    let flutter_views: FlutterViewList = vm_service
        .call_method(&methods.list_views())
        .map_err(handle_error)?;

    // Read stdin
    loop {
        let mut input = String::new();
        println!("🔥  To hot reload changes while running, press \"r\".");
        std::io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();
        let start = SystemTime::now();
        match trimmed {
            "r" => {
                // Send recompilation request to frontend server.
                let output = resident_compiler.recompile(
                        &std::path::Path::new("/Users/jonahwilliams/Documents/flutter/examples/flutter_gallery/lib/main.dart"),
                        devfs.updated_entries());
                 println!("Recompile Elapsed: {:?}", start.elapsed());
                // Write incremental dill file to devfs.
                vm_service
                    .write_file(
                        &mut std::fs::File::open(&output.output).unwrap(),
                        "lib/main.dart.incremental.dill",
                        "flutter_gallery",
                    )
                    .unwrap();
                 println!("Write File: {:?}", start.elapsed());

                // Call reload sources in each flutter view (generally only 1).
                // These should be sent concurrently if there are multiple.
                for view in flutter_views.views.iter() {
                    let base = &devfs_response.uri;
                    let report: ReloadReport = vm_service
                        .call_method(&methods.reload_sources(
                            &view.isolate.id,
                            false,
                            &format!("{}lib/main.dart.incremental.dill", base),
                            &format!("{}.packages", base),
                        ))
                        .unwrap();
                    println!("Reload sources: {:?}", start.elapsed());

                    // Inform compiler whether sources were accepted.
                    if report.success {
                        resident_compiler.accept();
                    } else {
                        resident_compiler.reject();
                    }
                    vm_service
                        .send_notification(&methods.reassemble(&view.isolate.id))
                        .unwrap();
                    println!("Reassemble: {:?}", start.elapsed());
                }
                println!("Total Elapsed: {:?}", start.elapsed());
            }
            _ => {}
        };
    }
}

fn handle_error<T>(_e: T) -> () {
    println!("Error");
}
