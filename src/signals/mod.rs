//! Signal handling utilities with RAII guards.
//!
//! Provides safe wrappers around libc signals with [`SignalKind`] for signal types
//! and [`SignalGuard`] for temporary signal handler management.

use std::collections::HashMap;

pub type SignalHandler = extern "C" fn(libc::c_int);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Safe wrapper around general libc signal.
pub struct SignalKind(libc::c_int);

macro_rules! impl_signal_delegates {
    (
        $(
            $constant:path = $name:ident
        ),*$(,)?
    ) => {
        $(
            #[doc = concat!("Wrapper around `", stringify!($constant), "`.")]
            pub const fn $name() -> Self {
                Self($constant)
            }
        )*
    };
}

impl SignalKind {
    /// Return raw [`c_int`](libc::c_int) stored in `self`.
    pub const fn as_raw(&self) -> libc::c_int {
        self.0
    }

    // C standard signals.
    // https://en.cppreference.com/w/c/program/SIG_types.html
    impl_signal_delegates!(
        libc::SIGABRT = abort,
        libc::SIGFPE = fpe,
        libc::SIGINT = int,
        libc::SIGILL = invalid,
        libc::SIGSEGV = segv,
        libc::SIGTERM = term,
    );

    // POSIX signals.
    // https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html
    #[cfg(unix)]
    impl_signal_delegates!(
        // libc::SIGABRT
        libc::SIGALRM = alarm,
        libc::SIGBUS = bus,
        libc::SIGCHLD = child,
        libc::SIGCONT = r#continue,
        // libc::SIGFPE
        libc::SIGHUP = hangup,
        // libc::SIGILL
        // libc::SIGINT
        libc::SIGKILL = kill,
        libc::SIGPIPE = pipe,
        libc::SIGQUIT = quit,
        // libc::SIGSEGV
        libc::SIGSTOP = stop,
        // libc::SIGTERM
        libc::SIGTSTP = terminal_stop,
        libc::SIGTTIN = tty_in,
        libc::SIGTTOU = tty_out,
        libc::SIGUSR1 = user1,
        libc::SIGUSR2 = user2,
        libc::SIGSYS = sys,
        libc::SIGTRAP = trap,
        libc::SIGURG = urgent,
        libc::SIGVTALRM = virtual_alarm,
        libc::SIGXCPU = xcpu,
        libc::SIGXFSZ = xfsz,
    );
}

impl From<SignalKind> for libc::c_int {
    fn from(value: SignalKind) -> Self {
        value.as_raw()
    }
}

impl From<libc::c_int> for SignalKind {
    fn from(value: libc::c_int) -> Self {
        Self(value)
    }
}

/// RAII guard for temporarily changing signal handlers.
/// Old handlers are restored on [`Drop`].
///
/// Built on top of [`libc::signal`].
pub struct SignalGuard {
    // SAFETY: For each entry holds, that `V` was created by `libc::signal(K, *new handler*)`.
    stashed_signals: HashMap<SignalKind, libc::sighandler_t>,
}

impl SignalGuard {
    /// Create [`SignalGuard`], which swaps signals from `signals` to [`SIG_IGN`](libc::SIG_IGN).
    pub fn ignore(signals: impl Iterator<Item = SignalKind>) -> Self {
        Self::new_impl_with_fallback(signals, None, libc::SIG_IGN as libc::sighandler_t)
    }

    /// Create [`SignalGuard`], which swaps signals from `signals` to [`SIG_DFL`](libc::SIG_DFL).
    pub fn default(signals: impl Iterator<Item = SignalKind>) -> Self {
        Self::new_impl_with_fallback(signals, None, libc::SIG_DFL as libc::sighandler_t)
    }

    fn new_impl_with_fallback(
        signals: impl Iterator<Item = SignalKind>,
        keys: Option<&HashMap<SignalKind, SignalHandler>>,
        fallback: libc::sighandler_t,
    ) -> Self {
        let get_signal_for = |kind| {
            let Some(keys) = keys else { return fallback };
            keys.get(&kind)
                // SAFETY: Since `handler` is `extern "C" fn(libc::c_int)`, therefore it's safe to
                // cast to C `void f(int)`, which is obscured by `libc::sighandler_t`.
                .map(|handler| *handler as libc::sighandler_t)
                .unwrap_or(fallback)
        };
        let mut stashed_signals = HashMap::new();
        for signal in signals {
            let new_handler = get_signal_for(signal);
            let old_handler = unsafe { libc::signal(signal.as_raw(), new_handler) };
            stashed_signals.insert(signal, old_handler);
        }
        Self { stashed_signals }
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        for (signal, action) in self.stashed_signals.iter() {
            // SAFETY: Since action was created by previous call to `libc::signal`, it's safe to
            // restore it.
            let _ = unsafe { libc::signal(signal.as_raw() as libc::c_int, *action) };
        }
    }
}
