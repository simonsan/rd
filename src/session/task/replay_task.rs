use crate::{
    arch::Architecture,
    kernel_abi::{common::preload_interface::syscallbuf_record, SupportedArch},
    registers::Registers,
    remote_ptr::{RemotePtr, Void},
    session::{
        task::{
            common::{
                did_waitpid,
                next_syscallbuf_record,
                open_mem_fd,
                read_bytes_fallible,
                read_bytes_helper,
                read_c_str,
                resume_execution,
                stored_record_size,
                syscallbuf_data_size,
                write_bytes,
                write_bytes_helper,
            },
            task_inner::{
                task_inner::{CloneReason, TaskInner, WriteFlags},
                CloneFlags,
                ResumeRequest,
                TicksRequest,
                WaitRequest,
            },
            Task,
        },
        Session,
    },
    trace::trace_frame::{FrameTime, TraceFrame},
    wait_status::WaitStatus,
};
use libc::pid_t;
use std::{
    ffi::CString,
    ops::{Deref, DerefMut},
};

pub struct ReplayTask {
    pub task_inner: TaskInner,
}

impl Deref for ReplayTask {
    type Target = TaskInner;

    fn deref(&self) -> &Self::Target {
        &self.task_inner
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ReplayTaskIgnore {
    IgnoreNone = 0,
    /// The x86 linux 3.5.0-36 kernel packaged with Ubuntu
    /// 12.04 has been observed to mutate $esi across
    /// syscall entry/exit.  (This has been verified
    /// outside of rr as well; not an rr bug.)  It's not
    /// clear whether this is a ptrace bug or a kernel bug,
    /// but either way it's not supposed to happen.  So we
    /// allow validate_args to cover up that bug.
    IgnoreEsi = 0x01,
}

impl ReplayTask {
    pub fn new(
        session: &dyn Session,
        tid: pid_t,
        rec_tid: pid_t,
        serial: u32,
        arch: SupportedArch,
    ) -> ReplayTask {
        ReplayTask {
            task_inner: TaskInner::new(session, tid, rec_tid, serial, arch),
        }
    }

    /// Initialize tracee buffers in this, i.e., implement
    /// RRCALL_init_syscall_buffer.  This task must be at the point
    /// of *exit from* the rrcall.  Registers will be updated with
    /// the return value from the rrcall, which is also returned
    /// from this call.  |map_hint| suggests where to map the
    /// region; see |init_syscallbuf_buffer()|.
    pub fn init_buffers(_map_hint: RemotePtr<Void>) {
        unimplemented!()
    }

    /// Call this method when the exec has completed.
    pub fn post_exec_syscall(&self, _replay_exe: &str) {
        unimplemented!()
    }

    /// Assert that the current register values match the values in the
    ///  current trace record.
    pub fn validate_regs(&self, _flags: ReplayTaskIgnore) {
        unimplemented!()
    }

    pub fn current_trace_frame(&self) -> &TraceFrame {
        unimplemented!()
    }

    pub fn current_frame_time(&self) -> FrameTime {
        unimplemented!()
    }

    /// Restore the next chunk of saved data from the trace to this.
    pub fn set_data_from_trace(&mut self) -> usize {
        unimplemented!()
    }

    /// Restore all remaining chunks of saved data for the current trace frame.
    pub fn apply_all_data_records_from_trace(&mut self) {
        unimplemented!()
    }

    /// Set the syscall-return-value register of this to what was
    /// saved in the current trace frame.
    pub fn set_return_value_from_trace(&mut self) {
        unimplemented!()
    }

    /// Used when an execve changes the tid of a non-main-thread to the
    /// thread-group leader.
    pub fn set_real_tid_and_update_serial(&mut self, _tid: pid_t) {
        unimplemented!()
    }

    /// Note: This method is private
    fn init_buffers_arch<Arch: Architecture>(_map_hint: RemotePtr<Void>) {
        unimplemented!()
    }
}

impl DerefMut for ReplayTask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task_inner
    }
}

impl Task for ReplayTask {
    /// Forwarded method
    fn resume_execution(
        &mut self,
        how: ResumeRequest,
        wait_how: WaitRequest,
        tick_period: TicksRequest,
        maybe_sig: Option<i32>,
    ) {
        resume_execution(self, how, wait_how, tick_period, maybe_sig)
    }

    /// Forwarded method
    fn stored_record_size(&mut self, record: RemotePtr<syscallbuf_record>) -> u32 {
        stored_record_size(self, record)
    }

    /// Forwarded method
    fn did_waitpid(&mut self, status: WaitStatus) {
        did_waitpid(self, status)
    }

    /// Forwarded method
    fn next_syscallbuf_record(&mut self) -> RemotePtr<syscallbuf_record> {
        next_syscallbuf_record(self)
    }

    fn as_task_inner(&self) -> &TaskInner {
        unimplemented!()
    }

    fn as_task_inner_mut(&mut self) -> &mut TaskInner {
        unimplemented!()
    }

    fn as_replay_task(&self) -> Option<&ReplayTask> {
        Some(self)
    }

    fn as_replay_task_mut(&mut self) -> Option<&mut ReplayTask> {
        Some(self)
    }

    fn on_syscall_exit(&self, _syscallno: i32, _arch: SupportedArch, _regs: &Registers) {
        unimplemented!()
    }

    fn at_preload_init(&self) {
        unimplemented!()
    }

    /// Forwarded method
    /// @TODO Forwarded method as this would be a non-overridden implementation
    fn clone_task(
        &self,
        _reason: CloneReason,
        _flags: CloneFlags,
        _stack: Option<RemotePtr<Void>>,
        _tls: Option<RemotePtr<Void>>,
        _cleartid_addr: Option<RemotePtr<i32>>,
        _new_tid: i32,
        _new_rec_tid: i32,
        _new_serial: u32,
        _other_session: Option<&dyn Session>,
    ) -> &TaskInner {
        unimplemented!()
    }

    /// Forwarded method
    fn open_mem_fd(&mut self) -> bool {
        open_mem_fd(self)
    }

    /// Forwarded method
    fn read_bytes_fallible(&mut self, addr: RemotePtr<u8>, buf: &mut [u8]) -> Result<usize, ()> {
        read_bytes_fallible(self, addr, buf)
    }

    /// Forwarded method
    fn read_bytes_helper(&mut self, addr: RemotePtr<Void>, buf: &mut [u8], ok: Option<&mut bool>) {
        read_bytes_helper(self, addr, buf, ok)
    }

    /// Forwarded method
    fn read_c_str(&mut self, child_addr: RemotePtr<u8>) -> CString {
        read_c_str(self, child_addr)
    }

    /// Forwarded method
    fn write_bytes_helper(
        &mut self,
        addr: RemotePtr<u8>,
        buf: &[u8],
        ok: Option<&mut bool>,
        flags: WriteFlags,
    ) {
        write_bytes_helper(self, addr, buf, ok, flags)
    }

    /// Forwarded method
    fn syscallbuf_data_size(&mut self) -> usize {
        syscallbuf_data_size(self)
    }

    /// Forwarded method
    fn write_bytes(&mut self, child_addr: RemotePtr<u8>, buf: &[u8]) {
        write_bytes(self, child_addr, buf);
    }
}