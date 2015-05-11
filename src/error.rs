extern crate rustc_serialize;

use std::error::Error as StdError;
use std::io::Error as IoError;
use hyper::Error as HttpError;
use url::ParseError;
use self::rustc_serialize::json::DecoderError;

use std::fmt::{Display, Formatter};
use std::fmt::Result as FormatResult;

use path::Expectation;

use self::Error::{
    InvalidUrl,
    MalformedRequest,
    RequestIoFailed,
    RequestFailed,
    DecodingFailed,
    ResponseParseFailed,
    QueryNotFinalized,
    QueryCompilationFailed,
    ExpectationNotSupported,
    VagueExpectation
};

#[derive(Debug)]
pub enum Error {
    InvalidUrl(ParseError, String),
    MalformedRequest(HttpError, String),
    RequestIoFailed(IoError, Vec<u8>),
    RequestFailed(HttpError, Vec<u8>),
    DecodingFailed(DecoderError, String),
    ResponseParseFailed,
    QueryNotFinalized,
    QueryCompilationFailed,
    ExpectationNotSupported(Expectation),
    VagueExpectation
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            InvalidUrl(_, _) => "Invalid Url",
            //InvalidUrl(_, ref url) => format!("Invalid Url: {}", &url).as_str(),
            MalformedRequest(_, _) => "Malformed Request",
            //MalformedRequest(_, ref url) => format!("Malformed Request, Url: {}", url).as_str(),
            RequestIoFailed(_, _) => "Request I/O Failed",
            //RequestIoFailed(_, path) => format!("Request I/O Failed, Path: {}", String::from_utf8(path).unwrap()).as_str(),
            RequestFailed(_, _) => "Request failed",
            //RequestFailed(_, path) => format!("Request Failed, Path: {}", String::from_utf8(path).unwrap()).as_str(),
            DecodingFailed(_, _) => "Decoding failed",
            //DecodingFailed(_, ref src) => format!("Decoding failed, Source: {:.200}", src).as_str(),
            ResponseParseFailed => "Response parsing failed",
            QueryNotFinalized => "Query is not finalized",
            QueryCompilationFailed => "Query compilation failed",
            ExpectationNotSupported(_) => "Finals like ToValue(), ToArray(), TagValue(), TagArray() are currently not supported in Cayley DB for HTTP queries and they return nothing.",
            VagueExpectation => "Driver has no knowledge of what to expect in response from Cayley"
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            InvalidUrl(ref perr, _) => Some(perr),
            MalformedRequest(ref herr, _) => Some(herr),
            RequestIoFailed(ref ioerr, _) => Some(ioerr),
            RequestFailed(ref herr, _) => Some(herr),
            DecodingFailed(ref derr, _) => Some(derr),
            _ => None,
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        InvalidUrl(err, "?".to_string())
    }
}

impl From<HttpError> for Error {
    fn from(err: HttpError) -> Error {
        RequestFailed(err, "?".to_string().into_bytes())
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        RequestIoFailed(err, "?".to_string().into_bytes())
    }
}

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Error {
        DecodingFailed(err, "?".to_string())
    }
}


impl From<Expectation> for Error {
    fn from(expectation: Expectation) -> Error {
        ExpectationNotSupported(expectation)
    }
}

//pub type GraphResult<T> = Result<T, RequestError>;
