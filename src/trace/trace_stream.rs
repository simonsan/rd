use crate::kernel_abi::SupportedArch;
use crate::remote_ptr::{RemotePtr, Void};
use crate::taskish_uid::TaskUid;
use crate::trace::trace_frame::FrameTime;
use crate::trace_capnp::Arch as TraceArch;
use crate::util::{dir_exists, ensure_dir};
use libc::pid_t;
use nix::errno::errno;
use nix::sys::stat::Mode;
use nix::unistd::mkdir;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;
use std::slice::Iter;

pub const TRACE_VERSION: u32 = 85;

pub const SUBSTREAM_COUNT: usize = 4;

/// Update `substreams` and TRACE_VERSION when you update this list.
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Substream {
    /// Substream that stores events (trace frames).
    Events = 0,
    RawData = 1,
    /// Substream that stores metadata about files mmap'd during
    /// recording.
    Mmaps = 2,
    /// Substream that stores task creation and exec events
    Tasks = 3,
}

/// This needs to be kept in sync with the enum above
pub const SUBSTREAMS: [Substream; SUBSTREAM_COUNT] = [
    Substream::Events,
    Substream::RawData,
    Substream::Mmaps,
    Substream::Tasks,
];

/// This needs to be kept in sync with the enum above
pub(super) const SUBSTREAMS_DATA: [SubstreamData; SUBSTREAM_COUNT] = [
    SubstreamData {
        name: "events",
        block_size: 1024 * 1024,
        threads: 1,
    },
    SubstreamData {
        name: "data",
        block_size: 1024 * 1024,
        // @TODO Hardcoded for now
        threads: 8,
    },
    SubstreamData {
        name: "mmaps",
        block_size: 64 * 1024,
        threads: 1,
    },
    SubstreamData {
        name: "tasks",
        block_size: 64 * 1024,
        threads: 1,
    },
];

pub(super) fn substream(s: Substream) -> &'static SubstreamData {
    // @TODO This method still needs to be completed
    &SUBSTREAMS_DATA[s as usize]
}

impl Substream {
    pub fn iter() -> Iter<'static, Substream> {
        SUBSTREAMS.iter()
    }
}

pub(super) struct SubstreamData {
    pub(super) name: &'static str,
    pub(super) block_size: usize,
    pub(super) threads: usize,
}

/// For REMAP_MAPPING maps, the memory contents are preserved so we don't
/// need a source. We use SourceZero for that case and it's ignored.
pub enum MappedDataSource {
    SourceTrace,
    SourceFile,
    SourceZero,
}

/// TraceStream stores all the data common to both recording and
/// replay.  TraceWriter deals with recording-specific logic, and
/// TraceReader handles replay-specific details.
/// writing code together for easier coordination.
impl TraceStream {
    /// Return the directory storing this trace's files.
    pub fn dir(&self) -> &OsStr {
        &self.trace_dir
    }

    pub fn bound_to_cpu(&self) -> i32 {
        self.bind_to_cpu
    }
    pub fn set_bound_cpu(&mut self, bound: i32) {
        self.bind_to_cpu = bound;
    }

    /// Return the current "global time" (event count) for this
    /// trace.
    pub fn time(&self) -> FrameTime {
        self.global_time
    }

    pub fn file_data_clone_file_name(&self, _tuid: &TaskUid) -> OsString {
        unimplemented!()
    }

    pub fn mmaps_block_size() -> usize {
        substream(Substream::Mmaps).block_size
    }

    pub(super) fn new(_trace_dir: &OsStr, _initial_time: FrameTime) -> TraceStream {
        unimplemented!()
    }

    /// Return the path of the file for the given substream.
    pub(super) fn path(&self, s: Substream) -> OsString {
        let mut path_vec: Vec<u8> = Vec::from(self.trace_dir.as_bytes());
        path_vec.extend_from_slice(b"/");
        path_vec.extend_from_slice(substream(s).name.as_bytes());
        OsString::from_vec(path_vec)
    }

    /// Return the path of "version" file, into which the current
    /// trace format version of rr is stored upon creation of the
    /// trace.
    pub(super) fn version_path(&self) -> OsString {
        let mut version_path: Vec<u8> = self.trace_dir.clone().into_vec();
        version_path.extend_from_slice(b"/version");
        OsString::from_vec(version_path)
    }

    /// While the trace is being built, the version file is stored under this name.
    /// When the trace is closed we rename it to the correct name. This lets us
    /// detect incomplete traces.
    pub(super) fn incomplete_version_path(&self) -> OsString {
        let mut version_path: Vec<u8> = self.trace_dir.clone().into_vec();
        version_path.extend_from_slice(b"/incomplete");
        OsString::from_vec(version_path)
    }

    /// Increment the global time and return the incremented value.
    pub(super) fn tick_time(&mut self) {
        self.global_time += 1
    }
}

/// TraceStream stores all the data common to both recording and
/// replay.  TraceWriter deals with recording-specific logic, and
/// TraceReader handles replay-specific details.
#[derive(Clone)]
pub struct TraceStream {
    /// Directory into which we're saving the trace files.
    pub(super) trace_dir: OsString,
    /// CPU core# that the tracees are bound to
    pub(super) bind_to_cpu: i32,
    /// Arbitrary notion of trace time, ticked on the recording of
    /// each event (trace frame).
    pub(super) global_time: FrameTime,
}

#[derive(Clone, Default)]
pub struct RawDataMetadata {
    pub addr: RemotePtr<Void>,
    pub size: usize,
    pub rec_tid: pid_t,
}

pub struct TraceRemoteFd {
    pub tid: pid_t,
    pub fd: i32,
}

/// Where to obtain data for the mapped region.
pub struct MappedData {
    pub time: FrameTime,
    pub source: MappedDataSource,
    /// Name of file to map the data from.
    pub filename: OsString,
    /// Data offset within `filename`.
    pub data_offset_bytes: usize,
    /// Original size of mapped file.
    pub file_size_bytes: usize,
}

pub(super) fn make_trace_dir(exe_path: &OsStr, output_trace_dir: &OsStr) -> OsString {
    if !output_trace_dir.is_empty() {
        // save trace dir in given output trace dir with option -o
        let ret = mkdir(output_trace_dir, Mode::S_IRWXU | Mode::S_IRWXG);
        if ret.is_ok() {
            return output_trace_dir.to_owned();
        }
        if libc::EEXIST == errno() {
            // directory already exists
            fatal!("Directory `{:?}' already exists.", output_trace_dir);
        } else {
            fatal!("Unable to create trace directory `{:?}'", output_trace_dir);
        }

        unreachable!()
    } else {
        // save trace dir set in _RD_TRACE_DIR or in the default trace dir
        ensure_dir(
            trace_save_dir().as_os_str(),
            "trace directory",
            Mode::S_IRWXU,
        );

        // Find a unique trace directory name.
        let mut nonce = 0;
        let mut ret;
        let mut dir;
        let mut ss: Vec<u8> = Vec::from(trace_save_dir().as_bytes());
        ss.extend_from_slice(b"/");
        ss.extend_from_slice(Path::new(exe_path).file_name().unwrap().as_bytes());
        loop {
            dir = Vec::from(ss.as_slice());
            write!(dir, "-{}", nonce).unwrap();
            nonce += 1;
            ret = mkdir(dir.as_slice(), Mode::S_IRWXU | Mode::S_IRWXG);
            if ret.is_ok() || libc::EEXIST != errno() {
                break;
            }
        }

        if ret.is_err() {
            fatal!("Unable to create trace directory `{:?}'", dir);
        }

        OsString::from_vec(dir)
    }
}

/// @TODO Look at logic again carefully
pub(super) fn default_rd_trace_dir() -> OsString {
    let cached_dir: OsString;
    let mut dot_dir: Vec<u8> = Vec::new();
    let maybe_home = env::var_os("HOME");
    let home: OsString;
    match maybe_home {
        Some(found_home) if !found_home.is_empty() => {
            dot_dir.extend_from_slice(found_home.as_bytes());
            dot_dir.extend_from_slice(b"/.rr");
            home = found_home;
        }
        // @TODO This seems to be an implicit outcome of what we have in rr
        _ => home = OsStr::from_bytes(b"").to_os_string(),
    }

    let mut xdg_dir: Vec<u8> = Vec::new();
    let maybe_xdg_data_home = env::var_os("XDG_DATA_HOME");
    match maybe_xdg_data_home {
        Some(xdg_data_home) if !xdg_data_home.is_empty() => {
            xdg_dir.extend_from_slice(xdg_data_home.as_bytes());
            xdg_dir.extend_from_slice(b"/rr");
        }
        _ => {
            xdg_dir.extend_from_slice(home.as_bytes());
            xdg_dir.extend_from_slice(b"/.local/share/rr");
        }
    }

    // If XDG dir does not exist but ~/.rr does, prefer ~/.rr for backwards
    // compatibility.
    if dir_exists(xdg_dir.as_slice()) {
        cached_dir = OsString::from_vec(xdg_dir);
    } else if dir_exists(dot_dir.as_slice()) {
        cached_dir = OsString::from_vec(dot_dir);
    } else if !xdg_dir.is_empty() {
        cached_dir = OsString::from_vec(xdg_dir);
    } else {
        cached_dir = OsStr::from_bytes(b"/tmp/rr").to_os_string();
    }

    cached_dir
}

pub(super) fn trace_save_dir() -> OsString {
    let maybe_output_dir = env::var_os("_RD_TRACE_DIR");
    match maybe_output_dir {
        Some(dir) if !dir.is_empty() => dir,
        _ => default_rd_trace_dir(),
    }
}

pub(super) fn latest_trace_symlink() -> OsString {
    let mut sym: Vec<u8> = Vec::from(trace_save_dir().as_bytes());
    sym.extend_from_slice(b"/latest-trace");
    OsString::from_vec(sym)
}

pub(super) fn to_trace_arch(arch: SupportedArch) -> TraceArch {
    match arch {
        SupportedArch::X86 => TraceArch::X86,
        SupportedArch::X64 => TraceArch::X8664,
    }
}
