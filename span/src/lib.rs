use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// An address into `SourceFiles`
#[derive(Clone, Copy)]
pub struct Offset(u32);

struct SourceFile {
    name: String,
    start: Offset,
    content: String,
}

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
    fn __new_source_file(&mut self, name: String, size: usize, content: String) -> Offset {
        let start = self.next_addr;
        self.next_addr.0 += TryInto::<u32>::try_into(size).unwrap();
        let src_file = SourceFile {
            name,
            start,
            content,
        };
        self.files.push(src_file);
        start
    }

    pub fn new_source_file(&mut self, name: String, content: String) -> Offset {
        self.__new_source_file(name, content.len(), content)
    }

    pub fn load_source_file(&mut self, path: &Path) -> Offset {
        let mut content = String::new();
        match __open_and_read(path, &mut content) {
            Result::Err(err) => panic!("load_source_file failed: {}", err),
            Result::Ok(size) => {
                self.__new_source_file(path.to_string_lossy().to_string(), size, content)
            }
        }
    }
}
