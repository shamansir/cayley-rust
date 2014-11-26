use url::ParseError;
use std::io::IoError;
use hyper::HttpError;
use serialize::json::DecoderError;

use std::fmt::{Show, Formatter};
use std::fmt::Result as FormatResult;

use path::Expectation;

pub enum RequestError {
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

// pub type Result = result::Result<(), Error>;

impl Show for RequestError {

    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        match *self {
            RequestError::InvalidUrl(ref perr, ref url) => {
                write!(f, "Invalid URL, Url(\"{}\"): ", url.as_slice());
                perr.fmt(f) },
            RequestError::MalformedRequest(ref herr, ref url) => {
                write!(f, "Malformed Request, Url(\"{}\"): ", url.as_slice());
                herr.fmt(f) },
            RequestError::RequestIoFailed(ref ioerr, ref path) => {
                write!(f, "Request I/O Failed, Path(\"{}\"): ", path.as_slice());
                ioerr.fmt(f) },
            RequestError::RequestFailed(ref herr, ref path) => {
                write!(f, "Request Failed, Path(\"{}\"): ", path.as_slice());
                herr.fmt(f) },
            RequestError::DecodingFailed(ref derr, ref src) => {
                write!(f, "Decoding Failed, Source(\"{}\"): ", src.as_slice());
                derr.fmt(f) },
            RequestError::ResponseParseFailed => "Response parsing failed".fmt(f),
            RequestError::QueryNotFinalized => "Query is not finalized".fmt(f),
            RequestError::QueryCompilationFailed => "Query can not be compiled".fmt(f),
            RequestError::ExpectationNotSupported(_) =>
                          "Finals like ToValue(), ToArray(), TagValue(), TagArray() are currently not supported in Cayley DB for HTTP queries and they return nothing.".fmt(f),
            RequestError::VagueExpectation =>
                          "Driver has no knowledge of what to expect in response from Cayley".fmt(f),
        }
    }
}

pub type GraphResult<T> = Result<T, RequestError>;
