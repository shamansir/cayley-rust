use url::ParseError;
use std::io::IoError;
use hyper::HttpError;
use std::fmt::{Show, Formatter, FormatError};
use serialize::json::DecoderError;

pub enum GraphRequestError {
    InvalidUrl(ParseError, String),
    MalformedRequest(HttpError, String),
    RequestIoFailed(IoError, String),
    RequestFailed(HttpError, String),
    DecodingFailed(DecoderError, String),
    ResponseParseFailed,
    QueryNotFinalized,
    QueryCompilationFailed
}

impl Show for GraphRequestError {

    #[allow(unused_must_use)]
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match *self {
            InvalidUrl(ref perr, ref url) => {
                write!(fmt, "Url(\"{}\"): ", url.as_slice());
                perr.fmt(fmt) },
            MalformedRequest(ref herr, ref url) => {
                write!(fmt, "Url(\"{}\"): ", url.as_slice());
                herr.fmt(fmt) },
            RequestIoFailed(ref ioerr, ref path) => {
                write!(fmt, "Path(\"{}\"): ", path.as_slice());
                ioerr.fmt(fmt) },
            RequestFailed(ref herr, ref path) => {
                write!(fmt, "Path(\"{}\"): ", path.as_slice());
                herr.fmt(fmt) },
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
