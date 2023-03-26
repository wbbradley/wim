pub type Buf = Vec<u8>;

macro_rules! buf_fmt {
    ($buf:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $buf.extend(formatted.as_bytes());
        formatted.len()
    }};
}
pub(crate) use buf_fmt;
