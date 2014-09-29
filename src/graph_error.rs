use url::ParseError;
use std::io::IoError;
use std::fmt::{Show, Formatter, FormatError};
use serialize::json::DecoderError;

pub enum GraphRequestError {
    InvalidUrl(ParseError, String),
    MalformedRequest(IoError, String),
    RequestFailed(IoError, String),
    DecodingFailed(DecoderError, String),
    ResponseParseFailed
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
            ResponseParseFailed => fmt.pad("Response parsing failed")
        }
    }
}

pub type GraphResult<T> = Result<T, GraphRequestError>;

/* pub enum GraphRequestErrorKind {
    InvalidUrl(ParseError),
    MalformedRequest(IoError),
    RequestFailed(IoError),
    DecodingFailed(DecoderError)
}

pub struct GraphRequestError {
    kind: GraphRequestErrorKind,
    message: SendStr
}

impl GraphRequestError {

    fn new<T: IntoMaybeOwned<'static>>(msg: T, kind: GraphRequestErrorKind) -> GraphRequestError {
        GraphRequestError {
            kind: kind,
            message: msg.into_maybe_owned()
        }
    }

    fn adapt_ioe_to_malformed(io: IoError) -> GraphRequestError {
        GraphRequestError::new(io.desc.into_maybe_owned(), MalformedRequest(io))
    }

    fn adapt_ioe_to_req_failed(io: IoError) -> GraphRequestError {
        GraphRequestError::new(io.desc.into_maybe_owned(), RequestFailed(io))
    }

    fn adapt_parse_to_invalid_url(pe: ParseError) -> GraphRequestError {
        GraphRequestError::new(pe.desc.into_maybe_owned(), InvalidUrl(io))
    }

    fn adapt_decode_to_dec_failed(de: DecoderError) -> GraphRequestError {
        GraphRequestError::new(de.desc.into_maybe_owned(), InvalidUrl(io))
    }

}

impl Show for GraphRequestError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match *self.kind {
            InvalidUrl(ref perr) => perr.fmt(fmt),
            MalformedRequest(ref ioerr) => ioerr.fmt(fmt),
            RequestFailed(ref ioerr) => ioerr.fmt(fmt),
            DecodingFailed(ref derr) => derr.fmt(fmt)
        }
    }
}

type GraphRequestResult = Result<Graph, GraphRequestError>; */
