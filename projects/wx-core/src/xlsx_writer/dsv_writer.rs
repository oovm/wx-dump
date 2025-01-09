use std::{borrow::Cow, fmt::Display, marker::PhantomData};

/// DSV 编码器
#[derive(Clone, Debug)]
pub struct DsvWriter {
    file_name: Cow<'static, str>,
}
impl DsvWriter {
    /// 新建一个 DSV 编码器
    pub fn new<T>(name: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self { file_name: name.into() }
    }
    // pub fn make_attachment<S>(&self, s: S) -> Attachment<Body>
    // where
    //     S: Stream<Item = Result<CsvLine, std::io::Error>> + Send + 'static,
    // {
    //     let sync = SyncStream::new(s).map_ok(|s| s.as_frame());
    //     let body = Body::from(BoxBody::new(http_body_util::StreamBody::new(sync)));
    //     Attachment::new(body).filename(self.file_name.to_string())
    // }
}
#[derive(Copy, Clone, Debug)]
pub struct CsvConfig;

impl DsvFormat for CsvConfig {
    const QUOTE: char = '"';
    const DELIMITER: char = ',';
    const ESCAPE: char = '"';
}

pub trait DsvFormat {
    const QUOTE: char;
    const DELIMITER: char;
    const ESCAPE: char;
}

pub type CsvLine = DsvLine<CsvConfig>;

#[derive(Clone, Debug)]
pub struct DsvLine<T> {
    buffer: String,
    config: PhantomData<T>,
}

impl<F: DsvFormat> DsvLine<F> {
    pub fn new() -> Self {
        Self { buffer: Default::default(), config: Default::default() }
    }
    pub fn needs_quote(&self, value: &str) -> bool {
        for c in value.chars() {
            if c == F::QUOTE || c == F::DELIMITER {
                return true;
            }
            if c.is_whitespace() {
                return true;
            }
        }
        false
    }
    pub fn push_display<T>(&mut self, value: T)
    where
        T: Display,
    {
        self.push_str(&format!("{}", value))
    }
    pub fn push_utf8_bom(&mut self) {
        self.buffer.push_str("\u{feff}");
    }
    pub fn push_str(&mut self, string: &str) {
        if self.needs_quote(string) {
            self.buffer.push(F::QUOTE);
            for c in string.chars() {
                if c == F::QUOTE {
                    self.buffer.push(F::ESCAPE);
                    self.buffer.push(F::QUOTE);
                }
                else {
                    self.buffer.push(c);
                }
            }
            self.buffer.push(F::QUOTE);
        }
        else {
            self.buffer.push_str(&string);
        }
        self.buffer.push(F::DELIMITER);
    }
    pub fn finish(mut self) -> String {
        self.buffer.push('\n');
        self.buffer
    }
    // #[allow(clippy::wrong_self_convention)]
    // pub fn as_bytes(self) -> Bytes {
    //     Bytes::from(self.finish().into_bytes())
    // }
    // #[allow(clippy::wrong_self_convention)]
    // pub fn as_frame(self) -> Frame<Bytes> {
    //     Frame::data(self.as_bytes().into())
    // }
}
