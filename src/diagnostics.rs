use codespan_reporting::{diagnostic::Diagnostic, files::Error, term::{termcolor::StandardStream}};
use color_eyre::{eyre::anyhow, Result};
use typst::{diag::SourceDiagnostic, syntax::FileId, World, WorldExt};

use crate::world::TypstWrapperWorld;

impl TypstWrapperWorld {
    fn into_codespan(&self, diag: &SourceDiagnostic) -> Result<Diagnostic<FileId>> {
        let span = diag.span;
        let message = diag.message.as_str().to_string();
        let codespan_diag = match diag.severity {
            typst::diag::Severity::Error => Diagnostic::error(),
            typst::diag::Severity::Warning => Diagnostic::warning(),
        }
        .with_message(&message)
        .with_label(codespan_reporting::diagnostic::Label {
            style: codespan_reporting::diagnostic::LabelStyle::Primary,
            file_id: span
                .id()
                .ok_or_else(|| anyhow!("Got a file-less diagnostic with message {}", message))?,
            range: self
                .range(span)
                .ok_or_else(|| anyhow!("Got a rangeless diagnostic with message {}", message))?,
            message: message,
        });
        Ok(codespan_diag)
    }
}

impl<'a> codespan_reporting::files::Files<'a> for TypstWrapperWorld {
    type FileId = FileId;

    type Name = String;

    type Source = String;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        Ok(id.vpath().as_rooted_path().to_string_lossy().to_string())
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        match self.file(id) {
            Ok(bytes) => {
                Ok(String::from_utf8(bytes.as_slice().to_vec()).map_err(|_| Error::FormatError)?)
            }
            Err(_) => Err(Error::FileMissing),
        }
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        match self.file(id) {
            Ok(f) => {
                let mut line = 0;
                for (i, b) in f.as_slice().iter().enumerate() {
                    if *b == b'\n' {
                        line += 1;
                    }
                    if i >= byte_index {
                        return Ok(line);
                    }
                }

                return Err(Error::IndexTooLarge {
                    given: byte_index,
                    max: line,
                });
            }
            Err(_) => Err(Error::FileMissing),
        }
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, Error> {
        match self.file(id) {
            Ok(f) => {
                let mut line = 0;
                let mut line_start = 0;
                for (i, b) in f.as_slice().iter().enumerate() {
                    if *b == b'\n' {
                        if line == line_index {
                            return Ok(line_start+1..i);
                        } else {
                            line += 1;
                            line_start = i;
                        }
                    }
                }

                return Err(Error::LineTooLarge {
                    given: line_index,
                    max: line,
                });
            }
            Err(_) => Err(Error::FileMissing),
        }
    }
}

pub struct Diagnostics {
    diagnostics: Vec<SourceDiagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
        }
    }

    pub fn push(&mut self, diag: SourceDiagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn report(&mut self, world: &TypstWrapperWorld) -> Result<()> {
        let color = codespan_reporting::term::termcolor::ColorChoice::Always;
        let mut writer = StandardStream::stderr(color);

        for diag in self.diagnostics.drain(..) {
            let diag = world.into_codespan(&diag)?;
            codespan_reporting::term::emit_to_write_style(
                &mut writer,
                &Default::default(),
                world,
                &diag,
            )
            .unwrap();
        }
        Ok(())
    }
}

impl Drop for Diagnostics {
    fn drop(&mut self) {
        if self.diagnostics.len() != 0 {
            println!("Dropped a Diagnostics that wasn't cleared")
        }
    }
}
