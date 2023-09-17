use super::RenderWidget;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Stdout};
use std::path::{Path, PathBuf};
use termion::raw::RawTerminal;
use tui::text::Text;
use tui::widgets::{Block, Borders, Paragraph};
use tui::{backend::TermionBackend, layout::Rect, Frame};

const DEFAULT_BUFFER_SIZE: u64 = 8 * 1024 * 4; // 4 KB

/// A widget that renders a text file's content onto the screen
pub struct TextFileViewer {
    file: PathBuf,
    buffer: String,
}

impl TextFileViewer {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Self {
        TextFileViewer {
            file: path.as_ref().to_path_buf(),
            buffer: String::with_capacity(DEFAULT_BUFFER_SIZE as usize),
        }
    }

    pub fn read(&mut self) -> io::Result<File> {
        let mut f = OpenOptions::new()
            .read(true)
            .write(false)
            .append(false)
            .create(false)
            .open(&self.file)?;
        f.read_to_string(&mut self.buffer)?;
        Ok(f)
    }
}

impl RenderWidget for TextFileViewer {
    fn render(&self, area: Rect, frame: &mut Frame<TermionBackend<RawTerminal<Stdout>>>) {
        let file_contents = Text::from(self.buffer.as_str());
        let paragraph = Paragraph::new(file_contents).block(
            Block::default()
                .title(self.file.display().to_string())
                .borders(Borders::all()),
        );
        frame.render_widget(paragraph, area);
    }
}
