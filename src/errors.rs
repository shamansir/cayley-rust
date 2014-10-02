use url::ParseError;
use std::io::IoError;
use std::fmt::{Show, Formatter, FormatError};
use serialize::json::DecoderError;

pub enum GraphRequestError {
    InvalidUrl(ParseError, String),
    MalformedRequest(IoError, String),
    RequestFailed(IoError, String),
    DecodingFailed(DecoderError, String),
    ResponseParseFailed,
    QueryNotFinalized,
    QueryCompilationFailed
}

impl Show for GraphRequestError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match *self {
            InvalidUrl(ref perr, ref url) => {
                write!(fmt, "Url(\"{}\"): ", url.as_slice());
                perr.fmt(fmt) },
            MalformedRequest(ref ioerr, ref url) => {
                write!(fmt, "Url(\"{}\"): ", url.as_slice());
                ioerr.fmt(fmt) },
            RequestFailed(ref ioerr, ref path) => {
                write!(fmt, "Path(\"{}\"): ", path.as_slice());
                ioerr.fmt(fmt) },
            DecodingFailed(ref derr, ref src) => {
                write!(fmt, "Source(\"{}\"): ", src.as_slice());
                derr.fmt(fmt) },
            ResponseParseFailed => fmt.pad("Response parsing failed"),
            QueryNotFinalized => fmt.pad("Query is not finalized"),
            QueryCompilationFailed => fmt.pad("Query can not be compiled")
        }
    }
}

pub type GraphResult<T> = Result<T, GraphRequestError>;
