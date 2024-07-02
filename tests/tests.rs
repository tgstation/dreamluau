#![allow(unused_must_use)]

use glob::glob;
use portpicker::pick_unused_port;
use std::{
    borrow::Cow,
    env,
    ffi::CStr,
    fmt::Display,
    fs,
    io::{Error, ErrorKind, Read, Result as IoResult, Write},
    net::{Ipv4Addr, TcpStream},
    ops::DerefMut,
    path::{PathBuf, MAIN_SEPARATOR_STR},
    process::{Child, Command},
    sync::{Once, OnceLock, RwLock, RwLockWriteGuard},
    thread::sleep,
    time::Duration,
};

fn byond_bin() -> PathBuf {
    PathBuf::from(std::env::var("BYOND_BIN").expect("environment variable BYOND_BIN"))
}

fn build() {
    let byond_bin = byond_bin();
    env::set_var(
        "CARGO_BUILD_TARGET",
        if cfg!(windows) {
            "i686-pc-windows-msvc"
        } else {
            "i686-unknown-linus-gnu"
        },
    );
    let dylib_path = test_cdylib::build_current_project()
        .to_str()
        .expect("target path is invalid UTF-8")
        .replace(MAIN_SEPARATOR_STR, "/");
    println!("{dylib_path}");
    let build_status = if cfg!(windows) {
        Command::new(byond_bin.join("dm.exe"))
    } else {
        let mut cmd = Command::new(byond_bin.join("byondexec"));
        cmd.arg(byond_bin.join("DreamMaker"));
        cmd
    }
    .arg(format!("-DDREAMLUAU=\"{dylib_path}\""))
    .arg(PathBuf::from_iter(["tests", "dm", "tests.dme"]))
    .status()
    .unwrap();
    if !build_status.success() {
        panic!("process exited with {:?}", build_status);
    }
}

fn free_port() -> &'static u16 {
    static FREE_PORT: OnceLock<u16> = OnceLock::new();
    FREE_PORT.get_or_init(|| pick_unused_port().expect("No open ports"))
}

fn topic_stream() -> RwLockWriteGuard<'static, TcpStream> {
    static TOPIC_STREAM: OnceLock<RwLock<TcpStream>> = OnceLock::new();
    TOPIC_STREAM
        .get_or_init(|| {
            let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, *free_port())).unwrap();
            RwLock::new(stream)
        })
        .write()
        .unwrap()
}
#[derive(Clone, Debug, PartialEq)]
enum TopicResponse {
    Null,
    Float(f32),
    String(String),
}

impl Display for TopicResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Float(fl) => f.write_fmt(format_args!("{fl}")),
            Self::String(s) => f.write_str(s),
        }
    }
}

fn send_topic<S: DerefMut<Target = TcpStream>>(
    stream: &mut S,
    topic: &str,
) -> IoResult<TopicResponse> {
    let len = topic.len() + 6;
    if len > u16::MAX as usize {
        return Err(Error::new(ErrorKind::Other, "payload size too large"));
    }
    stream.write_all(
        [0x00, 0x83]
            .into_iter()
            .chain((len as u16).to_be_bytes())
            .chain([0x00; 5])
            .chain(topic.bytes())
            .chain([0x00])
            .collect::<Vec<u8>>()
            .as_slice(),
    )?;
    stream.flush()?;
    let mut response = [0; 65535];
    let bytes_read = stream.read(&mut response[..])?;
    if bytes_read < 5 {
        return Err(Error::other("response too small"));
    }
    match response[4] {
        0x00 => Ok(TopicResponse::Null),
        0x2A => {
            if bytes_read > 9 {
                let mut float_bytes: [u8; 4] = Default::default();
                float_bytes.clone_from_slice(&response[5..9]);
                Ok(TopicResponse::Float(f32::from_be_bytes(float_bytes)))
            } else {
                Err(Error::other("response too small"))
            }
        }
        0x06 => {
            let mut length_bytes: [u8; 2] = Default::default();
            length_bytes.clone_from_slice(&response[2..4]);
            let string_length = (u16::from_be_bytes(length_bytes) - 1) as usize;
            if bytes_read < string_length + 5 {
                Err(Error::other("response too small"))
            } else {
                CStr::from_bytes_with_nul(&response[5..(5 + string_length)])
                    .map_err(Error::other)
                    .map(CStr::to_string_lossy)
                    .map(Cow::into_owned)
                    .map(TopicResponse::String)
            }
        }
        n => Err(Error::other(format! {"unexpected response type {n}"})),
    }
}

fn dreamdaemon() -> RwLockWriteGuard<'static, Child> {
    static DREAMDAEMON: OnceLock<RwLock<Child>> = OnceLock::new();
    static EXIT: Once = Once::new();
    EXIT.call_once(|| unsafe {
        extern "C" {
            fn atexit(cb: unsafe extern "C" fn());
        }
        extern "C" fn cleanup() {
            if let Some(Ok(mut dreamdaemon)) = DREAMDAEMON.get().map(RwLock::write) {
                dreamdaemon.kill();
            }
            #[cfg(debug_assertions)]
            for logfile in glob("tests/dm/meowtonin*.log").unwrap() {
                if let Ok(file) = logfile {
                    fs::remove_file(file);
                }
            }
            fs::remove_file("tests/dm/tests.dmb");
            fs::remove_file("tests/dm/tests.int");
        }
        atexit(cleanup);
    });
    DREAMDAEMON
        .get_or_init(|| {
            build();
            let byond_bin = byond_bin();
            let lock = RwLock::new(
                if cfg!(windows) {
                    Command::new(byond_bin.join("dd.exe"))
                } else {
                    let mut cmd = Command::new(byond_bin.join("byondexec"));
                    cmd.arg(byond_bin.join("DreamDaemon"));
                    cmd
                }
                .arg(PathBuf::from_iter(["tests", "dm", "tests.dmb"]))
                .arg(free_port().to_string())
                .arg("-trusted")
                .arg("-invisible")
                .spawn()
                .unwrap(),
            );
            sleep(Duration::from_secs_f32(3.0));
            lock
        })
        .write()
        .unwrap()
}

macro_rules! simple_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            dreamdaemon();
            let response = send_topic(
                &mut topic_stream(),
                format!("?test={}", stringify!($name)).as_str(),
            );
            assert_eq!(response.unwrap(), TopicResponse::Null)
        }
    };
}

simple_test!(hello_world);

simple_test!(usr_pushing);

simple_test!(calling);

simple_test!(sleeping);

simple_test!(yielding);

simple_test!(reading);

simple_test!(writing);

simple_test!(variants);
