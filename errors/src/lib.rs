use span::{Offset, SourceFiles, Span};
use std::io;
use std::io::Write;

pub enum Highlight {
    Point(Offset),
    Span(Span),
}

impl Highlight {
    #[inline]
    pub fn start(&self) -> Offset {
        match self {
            Highlight::Point(start) => *start,
            Highlight::Span(span) => span.start,
        }
    }

    #[inline]
    pub fn len(&self) -> Offset {
        match self {
            Highlight::Point(_) => Offset(1),
            Highlight::Span(span) => span.length,
        }
    }

    #[inline]
    pub fn end(&self) -> Offset {
        self.start().add(self.len().to_u32())
    }
}

pub struct Error {
    pub highlight: Highlight,
    pub message: String,
}

fn highlight<'src>(line: &'src str, line_offset: Offset, region: Highlight) -> String {
    let mut string = String::new();
    let mut pos: usize = 0;
    match region {
        Highlight::Point(offset) => {
            let offset = offset.to_usize() - line_offset.to_usize();
            for c in line.chars() {
                if pos == offset {
                    string.push('^');
                    break;
                } else {
                    string.push(' ');
                }
                pos += c.len_utf8();
            }
        }
        Highlight::Span(span) => {
            let mut in_range = false;
            for c in line.chars() {
                let line_offset = line_offset.to_usize();
                let start_offset = span.start.to_usize() - line_offset;
                let end_offset = span.end().to_usize() - line_offset;
                if in_range {
                    if pos == end_offset {
                        break;
                    } else {
                        string.push('^')
                    }
                } else {
                    if pos == start_offset {
                        in_range = true;
                        string.push('^')
                    } else {
                        string.push(' ')
                    }
                }
                pos += c.len_utf8();
            }
        }
    }
    string
}

pub fn __build_report(src_files: &SourceFiles, error: Error) -> [String; 5] {
    let error_start = error.highlight.start();
    let src_file = src_files.get_by_offset(error_start);
    let line = src_file.get_line(error_start);
    let highlight = highlight(line.content, line.offset, error.highlight);

    let line_number_string = line.number.to_string();
    let mut line_number_padding = String::new();
    for _ in line_number_string.chars() {
        line_number_padding.push(' ');
    }
    let line_number_padding = line_number_padding;

    let mut line0 = src_file.name.clone();
    line0 += "\n";

    let mut line1 = line_number_padding.clone();
    line1 += " |\n";

    let mut line2 = line_number_string;
    line2 += " | ";
    line2 += line.content;
    line2 += "\n";

    let mut line3 = line_number_padding.clone();
    line3 += " | ";
    line3 += &highlight;
    line3 += "\n";

    let mut line4 = String::from(error.message);
    line4 += "\n";

    [line0, line1, line2, line3, line4]
}

impl Error {
    pub fn report(self, src_files: &SourceFiles) {
        let [line0, line1, line2, line3, line4] = __build_report(src_files, self);
        let _ = io::stdout().write(line0.as_bytes()).unwrap();
        let _ = io::stdout().write(line1.as_bytes()).unwrap();
        let _ = io::stdout().write(line2.as_bytes()).unwrap();
        let _ = io::stdout().write(line3.as_bytes()).unwrap();
        let _ = io::stdout().write(line4.as_bytes()).unwrap();
    }
}

#[test]
fn test_build_report1() {
    let mut src_files = SourceFiles::new();
    src_files.new_source_file(
        String::from("test"),
        String::from("this is a line\nthis is another line"),
    );

    assert_eq!(
        __build_report(
            &src_files,
            Error {
                highlight: Highlight::Point(Offset(8)),
                message: String::from("Message")
            }
        ),
        [
            "test\n",
            "  |\n",
            "1 | this is a line\n",
            "  |         ^\n",
            "Message\n"
        ]
    )
}

#[test]
fn test_build_report2() {
    let mut src_files = SourceFiles::new();

    let mut prefix = String::from("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\nthis is ");
    let suffix = "another line";
    let aim = prefix.len();
    prefix += suffix;

    let content = prefix;

    src_files.new_source_file(String::from("test"), content);

    assert_eq!(
        __build_report(
            &src_files,
            Error {
                highlight: Highlight::Point(Offset(aim as u32)),
                message: String::from("Message")
            }
        ),
        [
            "test\n",
            "   |\n",
            "11 | this is another line\n",
            "   |         ^\n",
            "Message\n"
        ]
    )
}
