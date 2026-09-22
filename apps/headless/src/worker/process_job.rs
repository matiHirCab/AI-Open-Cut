//! Process-lifetime containment for a worker and all renderer descendants.
use opencut_editor_core::{CoreError, ErrorCode};
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, OwnedHandle};
use windows_sys::Win32::System::{
    JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    },
    Threading::GetCurrentProcess,
};

pub(super) fn install() -> Result<(), CoreError> {
    let failure = || {
        CoreError::new(
            ErrorCode::DependencyUnavailable,
            "cannot contain render worker process tree",
        )
    };
    // SAFETY: null security/name creates an unnamed, non-inheritable job handle.
    let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if handle.is_null() {
        return Err(failure());
    }
    // SAFETY: CreateJobObjectW returned a new owned handle.
    let job = unsafe { OwnedHandle::from_raw_handle(handle) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: the live handle, information class, initialized structure and its size agree.
    if unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            std::mem::size_of_val(&limits) as u32,
        )
    } == 0
    {
        return Err(failure());
    }
    // SAFETY: the pseudo-handle denotes this process; no breakaway flags are enabled.
    if unsafe { AssignProcessToJobObject(job.as_raw_handle(), GetCurrentProcess()) } == 0 {
        return Err(failure());
    }
    // Exactly one job is installed at CLI startup. Keep its non-inheritable handle
    // until OS process teardown (including crashes); closing it here kills us too.
    // OS teardown closes the last handle and terminates any surviving children.
    let _process_lifetime_handle = job.into_raw_handle();
    Ok(())
}
