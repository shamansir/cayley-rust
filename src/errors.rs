use url::ParseError;
use std::io::IoError;
use hyper::HttpError;
use serialize::json::DecoderError;

use std::fmt::{Show, Formatter};
use std::fmt::Error as FormatError;

use path::Expectation;

pub enum GraphRequestError {
    InvalidUrl(ParseError, String),
    MalformedRequest(HttpError, String),
    RequestIoFailed(IoError, String),
    RequestFailed(HttpError, String),
    DecodingFailed(DecoderError, String),
    ResponseParseFailed,
    QueryNotFinalized,
    QueryCompilationFailed,
    ExpectationNotSupported(Expectation),
    VagueExpectation
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
            QueryCompilationFailed => fmt.pad("Query can not be compiled"),
            ExpectationNotSupported(_) => fmt.pad("Finals like ToValue(), ToArray(), TagValue(), TagArray() are currently not supported in Cayley DB for HTTP queries and they return nothing."),
            VagueExpectation => fmt.pad("Driver has no knowledge of what to expect in response from Cayley"),
        }
    }
}

pub type GraphResult<T> = Result<T, GraphRequestError>;
