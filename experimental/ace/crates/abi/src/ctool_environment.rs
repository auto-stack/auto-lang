//! Windows environment snapshots sized before owning either UTF-16 scratch
//! or the lossless OsString. Keep the returned ticket with its actual owner.
use ace_source_query::resources::{Charge, Reservation, Resources};
use std::{ffi::OsString, os::windows::ffi::OsStringExt};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, ERROR_ENVVAR_NOT_FOUND},
    System::Environment::GetEnvironmentVariableW,
};

pub(crate) struct Value {
    pub(crate) value: OsString,
    pub(crate) allocation: Reservation,
}
pub(crate) fn read(name: &[u16], resources: &Resources) -> Result<Option<Value>, String> {
    if name.last() != Some(&0) {
        return Err("tool environment name is not terminated".into());
    }
    for _ in 0..4 {
        unsafe {
            SetLastError(0);
        }
        let size = unsafe { GetEnvironmentVariableW(name.as_ptr(), std::ptr::null_mut(), 0) };
        if size == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_ENVVAR_NOT_FOUND {
                return Ok(None);
            }
            if error != 0 {
                return Err(format!("tool environment size: {error}"));
            }
            return Ok(Some(Value {
                value: OsString::new(),
                allocation: resources
                    .reserve(Charge::index(0))
                    .map_err(|e| e.to_string())?,
            }));
        }
        // OsString's WTF-8 conversion may grow old/new buffers. Include its
        // full conversion peak and the still-live UTF-16 scratch, then refund
        // to the returned OsString's actual capacity after scratch is gone.
        let bound = (size as usize)
            .checked_mul(10)
            .and_then(|n| n.checked_add(128))
            .ok_or("tool environment size overflow")?;
        let mut allocation = resources
            .reserve(Charge::index(bound))
            .map_err(|e| e.to_string())?;
        let mut buffer = Vec::<u16>::new();
        buffer
            .try_reserve_exact(size as usize)
            .map_err(|e| format!("tool environment allocation: {e}"))?;
        buffer.resize(size as usize, 0);
        unsafe {
            SetLastError(0);
        }
        let written = unsafe { GetEnvironmentVariableW(name.as_ptr(), buffer.as_mut_ptr(), size) };
        if written >= size {
            continue;
        }
        if written == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_ENVVAR_NOT_FOUND {
                return Ok(None);
            }
            if error != 0 {
                return Err(format!("tool environment read: {error}"));
            }
        }
        let value = OsString::from_wide(&buffer[..written as usize]);
        drop(buffer);
        allocation
            .resize(Charge::index(value.capacity()))
            .map_err(|e| e.to_string())?;
        return Ok(Some(Value { value, allocation }));
    }
    Err("tool environment changed repeatedly during snapshot".into())
}
