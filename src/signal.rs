use std::io;

use libc;

unsafe fn init() -> io::Result<()> {
    let mut act: libc::sigaction = ::std::mem::zeroed();
    ok_errno((), libc::sigemptyset(&mut act.sa_mask))?;
    ok_errno((), libc::sigaddset(&mut act.sa_mask, libc::SIGWINCH))?;
    Ok(())
}

fn ok_errno<T>(ok: T, errcode: libc::c_int) -> io::Result<T> {
    if errcode != 0 {
        Err(io::Error::from_raw_os_error(errcode))
    } else {
        Ok(ok)
    }
}
