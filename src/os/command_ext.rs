use std::process::Command;
use std::io;

mod sealed {
    use std::process::Command;
    pub trait Sealed {}
    impl Sealed for Command {}
}

/// Adds portable [`exec_replace`](CommandExt::exec_replace) to the [`Command`].
pub trait CommandExt: sealed::Sealed {
    // TODO: Replace with `io::Result<!>`, when `!` is stabilized.
    /// Replace current process with command from `Self` and execute it.
    /// 
    /// # Returns
    /// [`Error`](io::Error) means, that spawning new command failed.
    /// Otherwise this function shall never return.
    fn exec_replace(&mut self) -> io::Error;
}

impl CommandExt for Command {
    #[cfg(unix)]
    fn exec_replace(&mut self) -> io::Error {
        use std::os::unix::process::CommandExt;
        self.exec()
    }

    #[cfg(windows)]
    fn exec_replace(&mut self) -> io::Error {
        use windows_sys::core::BOOL;
        use windows_sys::Win32::Foundation::{FALSE, TRUE};
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        unsafe extern "system" fn handler(_: u32) -> BOOL {
            TRUE
        }
        unsafe {
            if SetConsoleCtrlHandler(Some(handler), TRUE) == FALSE {
                return io::Error::other("failed to overwrite ctrl-c handler");
            }
        }
        let child_exit = {
            let child = match self.spawn() {
                Ok(child) => child,
                Err(e) => return e,
            };
            match child.wait() {
                Ok(status) => status,
                Err(e) => return e,
            }
        };
        std::process::exit(child_exit.code().unwrap_or(1))
    }

    #[cfg(not(any(unix, windows)))]
    fn exec_replace(&mut self) -> io::Error {
       io::Error::other("implement `exec_replace`")
    }
}
