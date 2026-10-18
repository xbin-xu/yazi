use std::{io, mem, os::windows::prelude::*, sync::Arc};

use windows_sys::Win32::{Globalization::CP_UTF8, System::Console::{CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, DISABLE_NEWLINE_AUTO_RETURN, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT, CTRL_C_EVENT, GetConsoleScreenBufferInfo, SetConsoleCP, SetConsoleCtrlHandler, SetConsoleMode, SetConsoleOutputCP}};
use yazi_shim::bool_ok;
use yazi_tty::Tty;

use crate::{Dimension, restorer::Restorer, source::EventSource};

pub struct Terminal<'a> {
	tty:          &'a Tty,
	pub source:   Arc<EventSource<'a>>,
	pub restorer: Restorer,
}

impl Drop for Terminal<'_> {
	fn drop(&mut self) { self.restorer.restore(self.tty); }
}

impl<'a> Terminal<'a> {
	pub fn new(tty: &'a Tty) -> io::Result<Self> {
		let source = Arc::new(EventSource::new(tty.reader())?);
		let restorer = Restorer::new(tty)?;

		let term = Self { tty, source, restorer };
		term.setup()?;

		Ok(term)
	}

	pub fn setup(&self) -> io::Result<()> {
		// Unix registers `SIGINT` and ignores it for the lifetime of the process; the
		// Windows equivalent is a console handler that discards `CTRL_C_EVENT`.
		//
		// `ENABLE_PROCESSED_INPUT` is what makes the console turn Ctrl-C into that event,
		// and it is back on whenever Yazi yields to a blocking child (`shell` with
		// `block=true`) — but yielding restores the console modes directly, without
		// going through a mode-switch method, so a handler tied to those switches would
		// never be installed for the child's lifetime. Installing once here covers every
		// path, and the default handler for the unhandled event is `ExitProcess`.
		// https://github.com/sxyazi/yazi/discussions/3180
		unsafe {
			SetConsoleCtrlHandler(Some(Self::ignore_ctrl_c), 1);
		}

		self.set_input_cp(CP_UTF8)?;
		self.set_output_cp(CP_UTF8)?;
		Ok(())
	}

	pub fn dimension(&self) -> Dimension {
		let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
		if unsafe { GetConsoleScreenBufferInfo(self.tty.writer().as_raw_handle(), &mut info) } == 0 {
			return Dimension::default();
		}

		Dimension {
			rows:   (info.srWindow.Bottom - info.srWindow.Top) as u16 + 1,
			cols:   (info.srWindow.Right - info.srWindow.Left) as u16 + 1,
			width:  0,
			height: 0,
		}
	}

	pub fn enter_raw_mode(&self) -> io::Result<()> {
		self.set_input_mode(
			(self.restorer.input_mode
				& !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
				| ENABLE_VIRTUAL_TERMINAL_INPUT
				| ENABLE_MOUSE_INPUT
				| ENABLE_WINDOW_INPUT,
		)?;

		self.set_output_mode(
			self.restorer.output_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN,
		)
	}

	pub fn enter_cooked_mode(&self) -> io::Result<()> {
		self.set_output_mode(
			self.restorer.output_mode
				& !(ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN),
		)?;

		self.set_input_mode(
			(self.restorer.input_mode
				& !(ENABLE_VIRTUAL_TERMINAL_INPUT | ENABLE_MOUSE_INPUT | ENABLE_WINDOW_INPUT))
				| ENABLE_ECHO_INPUT
				| ENABLE_LINE_INPUT
				| ENABLE_PROCESSED_INPUT,
		)
	}

	/// Handle and discard `CTRL_C_EVENT`, leaving every other event (Ctrl-Break,
	/// close, logoff, shutdown) to the default handler.
	unsafe extern "system" fn ignore_ctrl_c(ctrl_type: u32) -> i32 {
		(ctrl_type == CTRL_C_EVENT) as i32
	}

	fn set_input_cp(&self, cp: u32) -> io::Result<()> { bool_ok(unsafe { SetConsoleCP(cp) }) }

	fn set_output_cp(&self, cp: u32) -> io::Result<()> { bool_ok(unsafe { SetConsoleOutputCP(cp) }) }

	fn set_input_mode(&self, mode: CONSOLE_MODE) -> io::Result<()> {
		bool_ok(unsafe { SetConsoleMode(self.tty.reader().as_raw_handle(), mode) })
	}

	fn set_output_mode(&self, mode: CONSOLE_MODE) -> io::Result<()> {
		bool_ok(unsafe { SetConsoleMode(self.tty.writer().as_raw_handle(), mode) })
	}
}
