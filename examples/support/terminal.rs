#![allow(dead_code)]

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Stdout, Write};

pub struct TerminalSession {
    stdout: Stdout,
    mouse_capture: bool,
}

impl TerminalSession {
    pub fn new(mouse_capture: bool) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();

        if mouse_capture {
            execute!(
                &mut stdout,
                cursor::Hide,
                terminal::Clear(ClearType::All),
                EnableMouseCapture
            )?;
        } else {
            execute!(&mut stdout, cursor::Hide, terminal::Clear(ClearType::All))?;
        }

        Ok(Self {
            stdout,
            mouse_capture,
        })
    }

    pub fn present(&mut self, rendered: &str) -> io::Result<()> {
        execute!(&mut self.stdout, cursor::MoveTo(0, 0))?;
        write!(&mut self.stdout, "{}", rendered)?;
        self.stdout.flush()
    }

    fn restore(&mut self) -> io::Result<()> {
        if self.mouse_capture {
            execute!(
                &mut self.stdout,
                cursor::Show,
                terminal::Clear(ClearType::All),
                DisableMouseCapture
            )?;
        } else {
            execute!(
                &mut self.stdout,
                cursor::Show,
                terminal::Clear(ClearType::All)
            )?;
        }

        terminal::disable_raw_mode()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
