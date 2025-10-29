use std::process::Command;
use std::io;
use std::convert::Infallible;

mod sealed {
    use std::process::Command;
    pub trait Sealed {}
    impl Sealed for Command {}
}

/// Adds portable [`exec_replace`](CommandExt::exec_replace) to the [`Command`].
pub trait CommandExt: sealed::Sealed {
    // TODO: Replace `Infallible` with `!` when latter is stabilized.
    /// Replace current process with command from `Self` and execute it.
    /// 
    /// # Returns
    /// [`Err`](io::Error) variant means, that spawning new command failed.
    /// Otherwise this function shall never return.
    fn exec_replace(&mut self) -> io::Result<Infallible>;
}

impl CommandExt for Command {
    #[cfg(unix)]
    fn exec_replace(&mut self) -> io::Result<Infallible> {
        use std::os::unix::process::CommandExt;
        Err(self.exec())
    }

    #[cfg(windows)]
    fn exec_replace(&mut self) -> io::Result<Infallible> {
        use windows_sys::core::BOOL;
        use windows_sys::Win32::Foundation::{FALSE, TRUE};
        use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
        unsafe extern "system" fn handler(_: u32) -> BOOL {
            TRUE
        }
        unsafe {
            if SetConsoleCtrlHandler(Some(handler), TRUE) == FALSE {
                return Err(io::Error::other("failed to overwrite ctrl-c handler"));
            }
        }
        let status = self.spawn()?.wait()?;
        std::process::exit(status.code().unwrap_or(1))
    }

    #[cfg(not(any(unix, windows)))]
    fn exec_replace(&mut self) -> io::Result<Infallible> {
       Err(io::Error::other("implement `exec_replace`"))
    }
}
