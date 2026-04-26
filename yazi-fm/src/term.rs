use std::{io, ops::{Deref, DerefMut}, sync::atomic::{AtomicBool, Ordering}};

use anyhow::Result;
use crossterm::{event::{DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags}, execute, queue, style::Print, terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode}};
use ratatui::{CompletedFrame, Frame, Terminal, backend::CrosstermBackend, buffer::Buffer, layout::Rect};
use yazi_adapter::{Emulator, Mux, TMUX};
use yazi_config::{THEME, YAZI};
use yazi_shared::SyncCell;
use yazi_term::tty::{TTY, TtyWriter};

static CSI_U: AtomicBool = AtomicBool::new(false);

pub(super) struct Term {
	inner:       Terminal<CrosstermBackend<TtyWriter<'static>>>,
	last_area:   Rect,
	last_buffer: Buffer,
}

#[cfg(windows)]
unsafe extern "system" {
	fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
}

impl Term {
	pub(super) fn start() -> Result<Self> {
		let mut term = Self {
			inner:       Terminal::new(CrosstermBackend::new(TTY.writer()))?,
			last_area:   Default::default(),
			last_buffer: Default::default(),
		};

		#[cfg(windows)]
		Self::disable_ctrl_c();

		enable_raw_mode()?;
		static FIRST: SyncCell<bool> = SyncCell::new(false);
		if FIRST.replace(true) && yazi_adapter::TMUX.get() {
			yazi_adapter::Mux::tmux_passthrough();
		}

		execute!(
			TTY.writer(),
			yazi_term::If(!TMUX.get(), EnterAlternateScreen),
			Print("\x1bP$q q\x1b\\"), // Request cursor shape (DECRQSS query for DECSCUSR)
			Print("\x1b[?12$p"),      // Request cursor blink status (DECRQM query for DECSET 12)
			Print("\x1b[?u"),         // Request keyboard enhancement flags (CSI u)
			Print("\x1b[0c"),         // Request device attributes
			yazi_term::If(TMUX.get(), EnterAlternateScreen),
			yazi_term::SetBackground(true, THEME.app.bg_color()), // Set app background
			EnableBracketedPaste,
			EnableFocusChange,
			yazi_term::If(!YAZI.mgr.mouse_events.get().is_empty(), EnableMouseCapture),
		)?;

		let resp = Emulator::read_until_da1();
		Mux::tmux_drain()?;

		yazi_term::RestoreCursor::store(&resp);
		CSI_U.store(resp.contains("\x1b[?0u"), Ordering::Relaxed);
		if CSI_U.load(Ordering::Relaxed) {
			_ = queue!(
				TTY.writer(),
				PushKeyboardEnhancementFlags(
					KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
						| KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
				)
			);
		}

		if let Some(s) = YAZI.mgr.title() {
			queue!(TTY.writer(), SetTitle(s)).ok();
		}

		term.hide_cursor()?;
		term.clear()?;
		term.flush()?;
		Ok(term)
	}

	fn stop(&mut self) -> Result<()> {
		#[cfg(windows)]
		Self::enable_ctrl_c();

		if CSI_U.swap(false, Ordering::Relaxed) {
			queue!(TTY.writer(), PopKeyboardEnhancementFlags).ok();
		}

		if !YAZI.mgr.title_format.is_empty() {
			queue!(TTY.writer(), SetTitle("")).ok();
		}

		execute!(
			TTY.writer(),
			yazi_term::If(!YAZI.mgr.mouse_events.get().is_empty(), DisableMouseCapture),
			yazi_term::RestoreCursor,
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
		)?;

		self.show_cursor()?;
		Ok(disable_raw_mode()?)
	}

	pub(super) fn goodbye(f: impl FnOnce() -> i32) -> ! {
		if CSI_U.swap(false, Ordering::Relaxed) {
			queue!(TTY.writer(), PopKeyboardEnhancementFlags).ok();
		}

		if !YAZI.mgr.title_format.is_empty() {
			queue!(TTY.writer(), SetTitle("")).ok();
		}

		execute!(
			TTY.writer(),
			yazi_term::If(!YAZI.mgr.mouse_events.get().is_empty(), DisableMouseCapture),
			yazi_term::SetBackground(false, THEME.app.bg_color()),
			yazi_term::RestoreCursor,
			DisableFocusChange,
			DisableBracketedPaste,
			LeaveAlternateScreen,
			crossterm::cursor::Show
		)
		.ok();

		disable_raw_mode().ok();

		std::process::exit(f());
	}

	pub(super) fn draw(&mut self, f: impl FnOnce(&mut Frame)) -> io::Result<CompletedFrame<'_>> {
		let last = self.inner.draw(f)?;

		self.last_area = last.area;
		self.last_buffer = last.buffer.clone();
		Ok(last)
	}

	pub(super) fn draw_partial(
		&mut self,
		f: impl FnOnce(&mut Frame),
	) -> io::Result<CompletedFrame<'_>> {
		self.inner.draw(|frame| {
			let buffer = frame.buffer_mut();
			for y in self.last_area.top()..self.last_area.bottom() {
				for x in self.last_area.left()..self.last_area.right() {
					let mut cell = self.last_buffer[(x, y)].clone();
					cell.skip = false;
					buffer[(x, y)] = cell;
				}
			}

			f(frame);
		})
	}

	pub(super) fn can_partial(&mut self) -> bool {
		self.inner.autoresize().is_ok() && self.last_area == self.inner.get_frame().area()
	}

	/// On Windows, when a blocking shell subprocess is running, Ctrl+C generates
	/// `CTRL_C_EVENT` for all processes in the console group — including yazi itself.
	/// Without a custom handler, the default behavior terminates yazi, causing it to
	/// resume prematurely while the subprocess is still alive. This installs a handler
	/// that ignores `CTRL_C_EVENT`, letting only the subprocess handle it.
	#[cfg(windows)]
	fn enable_ctrl_c() {
		unsafe { SetConsoleCtrlHandler(Some(Self::ctrl_c_handler), 1) };
	}

	#[cfg(windows)]
	fn disable_ctrl_c() {
		unsafe { SetConsoleCtrlHandler(Some(Self::ctrl_c_handler), 0) };
	}

	#[cfg(windows)]
	unsafe extern "system" fn ctrl_c_handler(ctrl_type: u32) -> i32 {
		// CTRL_C_EVENT = 0 — ignore it so the blocking subprocess handles Ctrl+C alone
		if ctrl_type == 0 { 1 } else { 0 }
	}
}

impl Drop for Term {
	fn drop(&mut self) { self.stop().ok(); }
}

impl Deref for Term {
	type Target = Terminal<CrosstermBackend<TtyWriter<'static>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Term {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}
