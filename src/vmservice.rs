use libflate::gzip::Encoder;
use std::collections::HashMap;
use std::sync;
use std::sync::mpsc;

use websocket;

pub struct VMServiceMethods {
    id: i32,
}

impl VMServiceMethods {
    pub fn new() -> Self {
        VMServiceMethods { id: 0 }
    }

    // pub fn get_version<'a>(&mut self) -> VMServiceMethod<'a> {
    //     VMServiceMethod::GetVersion { id: self.next_id() }
    // }

    // pub fn get_vm<'a>(&mut self) -> VMServiceMethod<'a> {
    //     VMServiceMethod::GetVM { id: self.next_id() }
    // }

    pub fn reload_sources<'a>(
        &mut self,
        isolate_id: &'a str,
        pause: bool,
        root_lib_uri: &'a str,
        packages_uri: &'a str,
    ) -> VMServiceMethod<'a> {
        VMServiceMethod::ReloadSources {
            id: self.next_id(),
            params: ReloadSourcesParams {
                isolate_id: isolate_id,
                pause: pause,
                root_lib_uri: root_lib_uri,
                packages_uri: packages_uri,
            },
        }
    }

    pub fn create_devfs<'a>(&mut self, fs_name: &'a str) -> VMServiceMethod<'a> {
        VMServiceMethod::CreateDevFS {
            id: self.next_id(),
            params: CreateDevFSParams { fs_name: fs_name },
        }
    }

    // pub fn list_devfs<'a>(&mut self, fs_name: &'a str) -> VMServiceMethod<'a> {
    //     VMServiceMethod::ListDevFS {
    //         id: self.next_id(),
    //         params: ListDevFsParams { fs_name: fs_name },
    //     }
    // }

    pub fn list_views<'a>(&mut self) -> VMServiceMethod<'a> {
        VMServiceMethod::ListViews { id: self.next_id() }
    }

    pub fn register_service<'a>(
        &mut self,
        service: &'a str,
        alias: &'a str,
    ) -> VMServiceMethod<'a> {
        VMServiceMethod::RegisterService {
            id: self.next_id(),
            params: RegisterServiceParams {
                alias: alias,
                service: service,
            },
        }
    }

    // DANGER: DO NOT CALL ON DILL FILE.
    // pub fn write_devfs_file<'a>(
    //     &mut self,
    //     fs_name: &'a str,
    //     uri: &'a str,
    //     file: &mut std::fs::File,
    // ) -> VMServiceMethod<'a> {
    //     let mut buffer = Vec::new();
    //     file.read_to_end(&mut buffer).unwrap();
    //     VMServiceMethod::WriteDevFSFile {
    //         id: self.next_id(),
    //         params: WriteDevFSParams {
    //             fs_name: fs_name,
    //             uri: uri,
    //             file_contents: base64::encode(&buffer),
    //         },
    //     }
    // }

    pub fn delete_devfs<'a>(&mut self, fs_name: &'a str) -> VMServiceMethod<'a> {
        VMServiceMethod::DeleteDevFS {
            id: self.next_id(),
            params: DeleteDevFSParams { fs_name: fs_name },
        }
    }

    pub fn reassemble<'a>(&mut self, isolate_id: &'a str) -> VMServiceMethod<'a> {
        VMServiceMethod::Reassemble {
            id: self.next_id(),
            params: ReassembleParams {
                isolate_id: isolate_id,
            },
        }
    }

    fn next_id(&mut self) -> i32 {
        let next_id = self.id;
        self.id += 1;
        next_id
    }
}

/// Methods which can be called on the vm service.
#[derive(Debug, Serialize)]
#[serde(tag = "method")]
pub enum VMServiceMethod<'a> {
    #[serde(rename = "getVersion")]
    GetVersion { id: i32 },

    #[serde(rename = "getVM")]
    GetVM { id: i32 },

    #[serde(rename = "_flutter.listViews")]
    ListViews { id: i32 },

    #[serde(rename = "_reloadSources")]
    ReloadSources {
        id: i32,
        params: ReloadSourcesParams<'a>,
    },

    #[serde(rename = "_registerService")]
    RegisterService {
        id: i32,
        params: RegisterServiceParams<'a>,
    },

    #[serde(rename = "_createDevFS")]
    CreateDevFS {
        id: i32,
        params: CreateDevFSParams<'a>,
    },

    #[serde(rename = "_deleteDevFS")]
    DeleteDevFS {
        id: i32,
        params: DeleteDevFSParams<'a>,
    },

    #[serde(rename = "_writeDevFSFile")]
    WriteDevFSFile {
        id: i32,
        params: WriteDevFSParams<'a>,
    },
    #[serde(rename = "_listDevFS")]
    ListDevFS {
        id: i32,
        params: ListDevFsParams<'a>,
    },

    #[serde(rename = "ext.flutter.reassemble")]
    Reassemble {
        id: i32,
        params: ReassembleParams<'a>,
    },
}

impl<'a> VMServiceMethod<'a> {
    fn id(&self) -> i32 {
        match self {
            VMServiceMethod::GetVM { id } => *id,
            VMServiceMethod::GetVersion { id } => *id,
            VMServiceMethod::ListViews { id } => *id,
            VMServiceMethod::ReloadSources { id, .. } => *id,
            VMServiceMethod::RegisterService { id, .. } => *id,
            VMServiceMethod::CreateDevFS { id, .. } => *id,
            VMServiceMethod::WriteDevFSFile { id, .. } => *id,
            VMServiceMethod::DeleteDevFS { id, .. } => *id,
            VMServiceMethod::ListDevFS { id, .. } => *id,
            VMServiceMethod::Reassemble { id, .. } => *id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReassembleParams<'a> {
    #[serde(rename = "isolateId")]
    isolate_id: &'a str,
}

#[derive(Debug, Serialize)]
pub struct ListDevFsParams<'a> {
    #[serde(rename = "fsName")]
    fs_name: &'a str,
}

#[derive(Debug, Serialize)]
pub struct DeleteDevFSParams<'a> {
    #[serde(rename = "fsName")]
    fs_name: &'a str,
}

#[derive(Debug, Serialize)]
pub struct WriteDevFSParams<'a> {
    #[serde(rename = "fsName")]
    fs_name: &'a str,

    uri: &'a str,

    #[serde(rename = "fileContents")]
    file_contents: String,
}

#[derive(Debug, Serialize)]
pub struct ReloadSourcesParams<'a> {
    #[serde(rename = "isolateId")]
    isolate_id: &'a str,

    pause: bool,

    #[serde(rename = "rootLibUri")]
    root_lib_uri: &'a str,

    #[serde(rename = "packagesUri")]
    packages_uri: &'a str,
}

#[derive(Debug, Serialize)]
pub struct RegisterServiceParams<'a> {
    service: &'a str,
    alias: &'a str,
}

#[derive(Debug, Serialize)]
pub struct CreateDevFSParams<'a> {
    #[serde(rename = "fsName")]
    fs_name: &'a str,
}

/// The VMService is a handle to the dart vm service.
pub struct VMService {
    coordinator: sync::Arc<sync::Mutex<VMServiceCoordinator>>,
    http_address: String,
}

impl VMService {
    /// Connect to a vmservice at `addr` via a websocket.
    ///
    /// If successful, returns a handle to a [VMService].
    pub fn connect(addr: &str) -> Result<Self, ()> {
        // Connect to websocket vm service.
        let client = websocket::ClientBuilder::new(addr)
            .unwrap()
            .connect_insecure()
            .unwrap();

        // There is a way to do this without a mutex but I haven't spent time refactoing it.
        let (mut receiver, sender) = client.split().unwrap();
        let coordinator = sync::Arc::new(sync::Mutex::new(VMServiceCoordinator::new(sender)));
        let coordinator_ = sync::Arc::clone(&coordinator);

        // We expect a url of the form http://address:port/ws
        let mut http_address: String = addr.replace("ws://", "http://");
        http_address.truncate(http_address.len() - 2);

        // thread responsible for receiving messages from the vm service and
        // returning them to the main thread.
        std::thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message = message.expect("Failed to unwrap incoming message");
                let response: Response = match message {
                    websocket::OwnedMessage::Text(data) => serde_json::from_str(&data).unwrap(),
                    websocket::OwnedMessage::Ping(data) => serde_json::from_slice(&data).unwrap(),
                    websocket::OwnedMessage::Binary(data) => serde_json::from_slice(&data).unwrap(),
                    websocket::OwnedMessage::Pong(_) => {
                        continue;
                    }
                    websocket::OwnedMessage::Close(_) => {
                        continue;
                    }
                };
                let mut coordinator_ = coordinator.lock().unwrap();
                coordinator_.complete_job(response);
            }
        });
        let vmservice = VMService {
            coordinator: coordinator_,
            http_address: http_address,
        };
        Ok(vmservice)
    }

    /// Call a method on the vm service and block for the response.
    ///
    /// Requires an annotated type to infer the correct deserialziation.
    pub fn call_method<'a, T>(&mut self, method: &'a VMServiceMethod) -> Result<T, ()>
    where
        T: serde::de::DeserializeOwned,
    {
        let channel = {
            let mut coordinator = self.coordinator.lock().unwrap();
            coordinator.post_job(method)
        };
        match channel.recv().unwrap().result {
            Some(result) => Ok(serde_json::from_value(result).unwrap()),
            None => Err(()),
        }
    }

    /// Send a notification to the vm service without awaiting a response.
    pub fn send_notification<'a>(
        &mut self,
        method: &'a VMServiceMethod,
    ) -> Result<(), serde_json::Value> {
        let channel = {
            let mut coordinator_ = self.coordinator.lock().unwrap();
            coordinator_.post_job(method)
        };
        match channel.recv().unwrap().error {
            None => Ok(()),
            Some(err) => Err(err),
        }
    }

    pub fn send_unawaited<'a>(&mut self, method: &'a VMServiceMethod) {
        let mut coordinator = self.coordinator.lock().unwrap();
        coordinator.forget_job(method)
    }

    /// Writes a file to the vmservice devfs.
    ///
    /// In the case of a shared host vmservice, this could be replaced
    /// with a request that sends that absolute filepath to the dill.
    pub fn write_file<'a>(
        &mut self,
        content: &mut std::fs::File,
        device_uri: &'a str,
        fs_name: &'a str,
    ) -> Result<(), ()> {
        let client = reqwest::Client::new();
        let dev_fs_uri = base64::encode(device_uri.as_bytes());
        let mut encoder = Encoder::new(vec![]).unwrap();
        std::io::copy(content, &mut encoder).unwrap();
        let encoded_data = encoder.finish().into_result().unwrap();

        client
            .put(&self.http_address)
            .header("dev_fs_name", fs_name)
            .header("dev_fs_uri_b64", dev_fs_uri)
            .header("user-agent", "Dart/2.1 (dart:io)")
            .body(encoded_data)
            .send()
            .unwrap();
        Ok(())
    }
}

struct VMServiceCoordinator {
    pending: HashMap<i32, VMServiceJob>,
    sender: websocket::sender::Writer<std::net::TcpStream>,
}

impl VMServiceCoordinator {
    fn new(sender: websocket::sender::Writer<std::net::TcpStream>) -> Self {
        VMServiceCoordinator {
            pending: HashMap::new(),
            sender: sender,
        }
    }

    fn post_job(&mut self, method: &VMServiceMethod) -> mpsc::Receiver<Response> {
        let (sender, receiver) = mpsc::channel();
        let id = method.id();
        let job = VMServiceJob {
            id: id,
            sender: sender,
        };
        self.pending.insert(id, job);
        self.sender
            .send_message(&websocket::OwnedMessage::Text(
                serde_json::to_string(method).unwrap(),
            ))
            .unwrap();
        receiver
    }

    fn forget_job(&mut self, method: &VMServiceMethod) {
        self.sender
            .send_message(&websocket::OwnedMessage::Text(
                serde_json::to_string(method).unwrap(),
            ))
            .unwrap();
    }

    fn complete_job(&mut self, response: Response) {
        match self.pending.remove(&response.id) {
            Some(job) => job.sender.send(response).unwrap(),
            None => {}
        };
    }
}

struct VMServiceJob {
    id: i32,
    sender: mpsc::Sender<Response>,
}

/// VM Service events

#[derive(Debug, Copy, Clone)]
enum VMServiceEvents {
    VMUpdate,
    IsolateStart,
    IsolateRunnable,
    IsolateExit,
    IsolateUpdate,
    IsolateReload,
    IsolateSpawn,
    ServiceExtensionAdded,
    PauseStart,
    PauseExit,
    PauseBreakpoint,
    PauseInterrupted,
    PauseException,
    PausePostRequest,
    None,
    Resume,
    BreakpointAdded,
    BreakpointResolved,
    BreakpointRemoved,
    Graph,
    GC,
    Inspect,
    DebuggerSettingsUpdate,
    ConnectionClosed,
    Logging,
    Extension,
}

/// VM Service objects
///
/// See https://github.com/dart-lang/sdk/blob/master/runtime/vm/service/service.md

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: i32,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResult {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    /// The major version number is incremented when the protocol is changed
    /// in a potentially incompatible way.
    pub major: i64,

    /// The minor version number is incremented when the protocol is changed
    /// in a backwards compatible way.
    pub minor: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReloadReport {
    /// Whether the hot reload was successful.
    pub success: bool,
}

#[derive(Deserialize, Debug)]
pub struct DevFSListResult {
    #[serde(rename = "fsNames")]
    pub fs_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VM {
    /// Word length on target architecture (e.g. 32, 64).
    #[serde(rename = "architectureBits")]
    pub architecture_bits: i64,

    /// The CPU we are generating code for.
    #[serde(rename = "targetCPU")]
    pub target_cpu: String,

    /// The CPU we are actually running on.
    #[serde(rename = "hostCPU")]
    pub host_cpu: String,

    /// The Dart VM version string.
    pub version: String,

    /// The process id for the VM.
    pub pid: i64,

    /// The time that the VM started in milliseconds since the epoch.
    ///
    /// Suitable to pass to DateTime.fromMillisecondsSinceEpoch.
    #[serde(rename = "startTime")]
    pub start_time: i64,

    /// A list of isolates running in the VM.
    pub isolates: Vec<IsolateRef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IsolateRef {
    /// The id which is passed to the getIsolate RPC to load this isolate.
    pub id: String,

    /// A numeric id for this isolate, represented as a string. Unique.
    pub number: String,

    /// A name identifying this isolate. Not guaranteed to be unique.
    pub name: String,
}

/// FlutterView has a slightly different isolate type.
#[derive(Serialize, Deserialize, Debug)]
pub struct IsolateRef2 {
    /// The id which is passed to the getIsolate RPC to load this isolate.
    pub id: String,

    /// A numeric id for this isolate, represented as a string. Unique.
    pub number: i32,

    /// A name identifying this isolate. Not guaranteed to be unique.
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Isolate {
    /// The id which is passed to the getIsolate RPC to reload this
    /// isolate.
    pub id: String,

    /// A numeric id for this isolate, represented as a string. Unique.
    pub number: i32,

    /// A name identifying this isolate. Not guaranteed to be unique.
    pub name: String,

    /// The time that the VM started in milliseconds since the epoch.
    ///
    /// Suitable to pass to DateTime.fromMillisecondsSinceEpoch.
    #[serde(rename = "startTime")]
    pub start_time: i64,

    /// Is the isolate in a runnable state?
    pub runnable: bool,

    /// The number of live ports for this isolate.
    #[serde(rename = "livePorts")]
    pub live_ports: i64,

    /// Will this isolate pause when exiting?
    #[serde(rename = "pauseOnExit")]
    pub pause_on_exit: bool,

    /// The last pause event delivered to the isolate. If the isolate is
    /// running, this will be a resume event.
    #[serde(rename = "pauseEvent")]
    pub pause_event: Event,

    /// The root library for this isolate.
    ///
    /// Guaranteed to be initialized when the IsolateRunnable event fires.
    #[serde(rename = "rootLib")]
    pub root_lib: Option<LibraryRef>,

    /// A list of all libraries for this isolate.
    ///
    /// Guaranteed to be initialized when the IsolateRunnable event fires.
    pub libraries: Vec<LibraryRef>,

    /// A list of all breakpoints for this isolate.
    pub breakpoints: Vec<Breakpoint>,

    /// The error that is causing this isolate to exit, if applicable.
    pub error: Option<IsolateError>,

    /// The current pause on exception mode for this isolate.
    #[serde(rename = "exceptionPauseMode")]
    pub exception_pause_mode: ExceptionPauseMode,

    /// The list of service extension RPCs that are registered for this isolate,
    /// if any.
    #[serde(rename = "extensionRPCs")]
    pub extension_rpcs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryRef {
    /// The name of this library.
    pub name: String,

    /// The uri of this library.
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    /// The name of this library.
    pub name: String,

    /// The uri of this library.
    pub uri: String,

    /// Is this library debuggable? Default true.
    pub debuggable: bool,

    /// A list of the imports for this library.
    pub dependencies: Vec<LibraryDependency>,

    /// A list of the scripts which constitute this library.
    pub scripts: Vec<ScriptRef>,

    /// A list of the top-level variables in this library.
    pub variables: Vec<FieldRef>,

    /// A list of the top-level functions in this library.
    pub functions: Vec<FunctionRef>,

    /// A list of all classes in this library.
    pub classes: Vec<ClassRef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlutterViewList {
    pub views: Vec<FlutterView>,
}

#[derive(Deserialize, Debug)]
pub struct DevFSCreatedResponse {
    /// The root uri of the created devfs.
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlutterView {
    pub id: String,

    /// The main UI isolate.
    pub isolate: IsolateRef2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryDependency {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptRef {}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldRef {}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionRef {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClassRef {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExceptionPauseMode {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Breakpoint {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {}

#[derive(Serialize, Deserialize, Debug)]
pub struct IsolateError {}
