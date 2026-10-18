/// Discard `CTRL_C_EVENT` for the lifetime of the process, mirroring the way the Unix
/// build ignores `SIGINT`: the interrupt must reach the blocking child alone, or it and
/// Yazi both act on it and end up fighting over stdin.
///
/// https://github.com/sxyazi/yazi/discussions/3180
#[cfg(windows)]
pub fn ignore_ctrl_c() -> anyhow::Result<()> {
	use windows_sys::Win32::System::Console::{CTRL_C_EVENT, SetConsoleCtrlHandler};
	use yazi_shim::bool_ok;

	unsafe extern "system" fn handle(ctrl_type: u32) -> i32 {
		(ctrl_type == CTRL_C_EVENT) as i32
	}

	Ok(bool_ok(unsafe { SetConsoleCtrlHandler(Some(handle), 1) })?)
}
