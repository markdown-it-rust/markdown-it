pub use sourcepos::SourcePos;

#[cfg(feature="sourcemap")]
pub use charmapping::CharMapping;

#[cfg(feature="sourcemap")]
mod charmapping {
    #[derive(Debug)]
    pub struct CharMapping {
        src: String,
        marks: Vec<CharMappingMark>,
    }

    impl CharMapping {
        pub fn new(src: &str) -> Self {
            let mut iterator = src.char_indices().peekable();
            let mut line = 1;
            let mut column = 0;
            let mut marks = vec![CharMappingMark { offset: 0, line, column }];

            loop {
                match iterator.next() {
                    Some((_, '\r')) if matches!(iterator.peek(), Some((_, '\n'))) => {
                        // ignore \r followed by \n
                        column += 1;
                    }
                    Some((offset, '\r' | '\n')) => {
                        // \r or \n are linebreaks
                        line += 1;
                        column = 0;
                        marks.push(CharMappingMark { offset: offset + 1, line, column });
                    }
                    Some((offset, _)) => {
                        // any other character, just increase position
                        if column % 16 == 0 && column > 0 {
                            marks.push(CharMappingMark { offset, line, column });
                        }
                        column += 1;
                    },
                    None => break,
                }
            }

            Self { src: src.to_owned(), marks }
        }

        pub(super) fn get_position(&self, byte_offset: usize) -> (u32, u32) {
            let byte_offset = byte_offset + 1; // include current char
            let found = match self.marks.binary_search_by(|mark| mark.offset.cmp(&byte_offset)) {
                Ok(x) => x,
                Err(x) => x - 1,
            };
            let mark = &self.marks[found];
            let line = mark.line;
            let mut column = mark.column;
            for (offset, _) in self.src[mark.offset..].char_indices() {
                if mark.offset + offset >= byte_offset { break; }
                column += 1;
            }
            (line, column)
        }
    }

    #[derive(Debug)]
    struct CharMappingMark {
        offset: usize,
        line: u32,
        column: u32,
    }
}

mod sourcepos {
    #[cfg(feature="sourcemap")]
    use super::CharMapping;

    #[derive(Default, Clone, Copy)]
    pub struct SourcePos {
        #[cfg(feature="sourcemap")]
        byte_offset: (usize, usize),
        __private: (),
    }

    impl SourcePos {
        pub fn new(_start: usize, _end: usize) -> Self {
            SourcePos {
                #[cfg(feature="sourcemap")]
                byte_offset: (_start, _end),
                __private: (),
            }
        }

        #[cfg(feature="sourcemap")]
        pub fn get_byte_offsets(&self) -> (usize, usize) {
            self.byte_offset
        }

        #[cfg(feature="sourcemap")]
        /// Returns (line_start, column_start, line_end, column_end)
        pub fn get_positions(&self, map: &CharMapping) -> ((u32, u32), (u32, u32)) {
            let start = map.get_position(self.byte_offset.0);
            let end_off = if self.byte_offset.1 > 0 { self.byte_offset.1 - 1 } else { self.byte_offset.1 };
            let end = map.get_position(end_off);
            (start, end)
        }
    }

    impl std::fmt::Debug for SourcePos {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            #[cfg(feature="sourcemap")]
            return self.byte_offset.fmt(f);
            #[cfg(not(feature="sourcemap"))]
            return self.__private.fmt(f);
        }
    }

    #[cfg(test)]
    #[cfg(feature="sourcemap")]
    mod tests {
        use super::CharMapping;
        use super::SourcePos;

        #[test]
        fn no_linebreaks() {
            let map = CharMapping::new("qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM");
            for i in 0..20 {
                assert_eq!(SourcePos::new(i, 0).get_positions(&map).0, (1, i as u32 + 1));
            }
        }

        #[test]
        fn unicode() {
            let map = CharMapping::new("!ΑαΒβΓγΔδΕεΖζΗηΘθΙιΚκΛλΜμΝνΞξΟοΠπΡρΣσςΤτΥυΦφΧχΨψΩω");
            assert_eq!(SourcePos::new(0, 0).get_positions(&map).0, (1, 1));
            for i in 1..20 {
                assert_eq!(SourcePos::new(i, 0).get_positions(&map).0, (1, ((i - 1) / 2) as u32 + 2));
            }
        }

        #[test]
        fn many_linebreaks() {
            let map = CharMapping::new("\n\n\n\n\n\n123");
            for i in 0..6 {
                assert_eq!(SourcePos::new(i, 0).get_positions(&map).0, (i as u32 + 2, 0));
            }
            assert_eq!(SourcePos::new(7, 0).get_positions(&map).0, (7, 2));
            assert_eq!(SourcePos::new(8, 0).get_positions(&map).0, (7, 3));
        }

        #[test]
        fn after_end() {
            let map = CharMapping::new("123");
            assert_eq!(SourcePos::new(100, 0).get_positions(&map).0, (1, 3));
            let map = CharMapping::new("123\n");
            assert_eq!(SourcePos::new(100, 0).get_positions(&map).0, (2, 0));
            let map = CharMapping::new("123\n456");
            assert_eq!(SourcePos::new(100, 0).get_positions(&map).0, (2, 3));
        }
    }
}
