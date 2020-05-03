use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// An address into `SourceFiles`
#[derive(Clone, PartialOrd, Ord, Copy, Debug, PartialEq, Eq)]
pub struct Offset(pub u32);

impl Offset {
    #[inline]
    pub fn add_mut(&mut self, n: u32) {
        self.0 += n
    }

    #[inline]
    pub fn add(self, n: u32) -> Self {
        Offset(self.0 + n)
    }

    #[inline]
    pub fn subtract(self, n: u32) -> Self {
        Offset(self.0 - n)
    }

    #[inline]
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: Offset,
    pub length: Offset,
}

impl Span {
    #[inline]
    pub fn end(&self) -> Offset {
        self.start.add(self.length.to_u32())
    }
}

#[derive(Debug)]
/// `SourceFile` is exposed for testing, but these should generally be obtained by reference using
/// `SourceFiles`
pub struct SourceFile {
    pub name: String,
    pub start: Offset,
    pub content: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Line<'src> {
    /// `Offset` of the beginning of the line
    pub offset: Offset,
    /// Line number in the file
    pub number: u32,
    /// Line contents
    pub content: &'src str,
}

fn is_newline(c: &char) -> bool {
    match c {
        '\n' => true,
        _ => false,
    }
}

impl SourceFile {
    #[inline]
    pub fn get_start(&self) -> Offset {
        self.start
    }

    pub fn get_line(&self, offset: Offset) -> Line {
        let offset = offset.subtract(self.start.to_u32());
        let mut pos: usize = 0;

        let mut line_start = 0;
        let mut line_end = 0;

        let mut number = 1;
        let content = self.content.as_str();

        let mut found = false;
        for ref c in content.chars() {
            if found {
                pos += {
                    if is_newline(c) {
                        0
                    } else {
                        c.len_utf8()
                    }
                };

                line_end = pos;

                if is_newline(c) {
                    break;
                }
            } else {
                if pos >= offset.to_usize() {
                    found = true;
                }

                pos += c.len_utf8();

                if is_newline(c) {
                    number += 1;
                    line_start = pos;
                }
            }
        }

        if found {
            Line {
                offset: self.start.add(line_start.try_into().unwrap()),
                number,
                content: &content[line_start..line_end],
            }
        } else {
            panic!("get_line: no line containing {:?}", offset)
        }
    }
}

impl SourceFile {
    pub fn data<'src>(&'src self) -> &'src str {
        &self.content
    }
}

#[derive(Debug)]
pub struct SourceFiles {
    next_addr: Offset,
    files: Vec<SourceFile>,
}

#[inline]
fn __open_and_read(path: &Path, mut content: &mut String) -> std::io::Result<usize> {
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)
}

impl SourceFiles {
    #[inline]
    pub fn new() -> Self {
        SourceFiles {
            next_addr: Offset(0),
            files: Vec::new(),
        }
    }

    #[inline]
    fn __new_source_file(
        &mut self,
        name: String,
        size: usize,
        content: String,
    ) -> (Offset, String) {
        let start = self.next_addr;
        self.next_addr = start.add(size.try_into().unwrap());
        let name_copy = name.clone();
        let src_file = SourceFile {
            name,
            start,
            content,
        };
        self.files.push(src_file);
        (start, name_copy)
    }

    pub fn new_source_file(&mut self, name: String, content: String) -> Offset {
        self.__new_source_file(name, content.len(), content).0
    }

    pub fn load_source_file<'files>(&'files mut self, path: &Path) -> (Offset, String) {
        let mut content = String::new();
        match __open_and_read(path, &mut content) {
            Result::Err(err) => panic!("load_source_file failed: {}", err),
            Result::Ok(size) => {
                self.__new_source_file(path.to_string_lossy().to_string(), size, content)
            }
        }
    }

    pub fn get_by_offset<'src>(&'src self, offset: Offset) -> &'src SourceFile {
        if offset >= self.next_addr {
            panic!("get_by_offset failed: offset out of bounds")
        }
        let ix = match self.files.binary_search_by_key(&offset, |file| file.start) {
            Result::Ok(ix) => ix,
            Result::Err(ix) => ix - 1,
        };
        &self.files[ix]
    }

    pub fn get_by_name<'src>(&'src self, name: &str) -> &'src SourceFile {
        for file in self.files.iter() {
            if file.name == name {
                return &file;
            }
        }
        panic!("get_by_name failed: no name {:?} found", name)
    }
}

#[test]
fn test_get_by_offset1() {
    let mut src_files = SourceFiles::new();

    let content_one = String::from("some letters");
    src_files.new_source_file(String::from("one"), content_one.clone());
    println!("{:?}", src_files);

    let content_two = String::from("content");
    src_files.new_source_file(String::from("two"), content_two.clone());
    println!("{:?}", src_files);

    let content_three = String::from("other letters");
    src_files.new_source_file(String::from("three"), content_three.clone());
    println!("{:?}", src_files);

    assert_eq!(
        src_files.get_by_offset(Offset(0)).data(),
        content_one.clone()
    );
    assert_eq!(
        src_files.get_by_offset(Offset(1)).data(),
        content_one.clone()
    );
    assert_eq!(
        src_files.get_by_offset(Offset(11)).data(),
        content_one.clone()
    );

    assert_eq!(
        src_files.get_by_offset(Offset(12)).data(),
        content_two.clone()
    );
    assert_eq!(
        src_files.get_by_offset(Offset(14)).data(),
        content_two.clone()
    );
    assert_eq!(
        src_files.get_by_offset(Offset(18)).data(),
        content_two.clone()
    );

    assert_eq!(
        src_files.get_by_offset(Offset(19)).data(),
        content_three.clone()
    );
    assert_eq!(
        src_files.get_by_offset(Offset(24)).data(),
        content_three.clone()
    );
    assert_eq!(
        src_files.get_by_offset(Offset(31)).data(),
        content_three.clone()
    );
}

#[test]
fn test_get_line1() {
    let src_file = SourceFile {
        name: String::from("test"),
        start: Offset(0),
        content: String::from("hello"),
    };
    assert_eq!(
        src_file.get_line(Offset(0)),
        Line {
            offset: Offset(0),
            number: 1,
            content: "hello"
        }
    )
}

#[test]
fn test_get_line2() {
    let src_file = SourceFile {
        name: String::from("test"),
        start: Offset(0),
        content: String::from("hello\n"),
    };
    assert_eq!(
        src_file.get_line(Offset(0)),
        Line {
            offset: Offset(0),
            number: 1,
            content: "hello"
        }
    )
}

#[test]
fn test_get_line3() {
    let src_file = SourceFile {
        name: String::from("test"),
        start: Offset(2),
        content: String::from("hello"),
    };
    assert_eq!(
        src_file.get_line(Offset(4)),
        Line {
            offset: Offset(2),
            number: 1,
            content: "hello"
        }
    )
}

#[test]
fn test_get_line4() {
    let src_file = SourceFile {
        name: String::from("test"),
        start: Offset(5),
        content: String::from("hello\nworld"),
    };
    assert_eq!(
        src_file.get_line(Offset(11)),
        Line {
            offset: Offset(11),
            number: 2,
            content: "world"
        }
    )
}

#[test]
fn test_get_line5() {
    let src_file = SourceFile {
        name: String::from("test"),
        start: Offset(5),
        content: String::from("hello\nworld\nyay"),
    };
    assert_eq!(
        src_file.get_line(Offset(11)),
        Line {
            offset: Offset(11),
            number: 2,
            content: "world"
        }
    )
}

#[test]
fn test_get_line6() {
    let src_file = SourceFile {
        name: String::from("test"),
        start: Offset(5),
        content: String::from("hello\nworld"),
    };
    assert_eq!(
        src_file.get_line(Offset(14)),
        Line {
            offset: Offset(11),
            number: 2,
            content: "world"
        }
    )
}
