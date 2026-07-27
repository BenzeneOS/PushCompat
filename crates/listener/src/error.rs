use std::{
   error,
   fmt::{
      Display,
      Formatter,
      Result as FmtResult,
   },
   io::Error as IoError,
};

#[derive(Debug)]
pub enum Error {
   /// Dependency failed, i.e. we blame them.
   DependencyFailure(&'static str, &'static str),
   /// Dependency rejection, i.e. they blame us.
   DependencyRejection(&'static str, String),
   /// Protobuf deserialization failure, probably a contract change.
   ProtobufDecode(&'static str, quick_protobuf::Error),
   Request(&'static str, reqwest::Error),
   Response(&'static str, reqwest::Error),
   Socket(IoError),
}

impl Display for Error {
   fn fmt(&self, f: &mut Formatter) -> FmtResult {
      match self {
         Self::DependencyFailure(api, problem) => write!(f, "{api} API {problem}"),
         Self::DependencyRejection(api, reason) => {
            write!(f, "{api} API rejected request: {reason}")
         },
         Self::ProtobufDecode(kind, error) => write!(f, "Error decoding {kind}: {error}"),
         Self::Request(kind, error) => write!(f, "{kind} API request error: {error}"),
         Self::Response(kind, error) => write!(f, "{kind} API response error: {error}"),
         Self::Socket(error) => write!(f, "TCP error: {error}"),
      }
   }
}

impl error::Error for Error {
   fn description(&self) -> &str {
      match self {
         Self::DependencyFailure(_, reason) => reason,
         Self::DependencyRejection(_, reason) => reason,
         Self::ProtobufDecode(..) => "protobuf deserialization failed",
         Self::Request(..) => "request failed",
         Self::Response(..) => "response failed",
         Self::Socket(_) => "socket operation failed",
      }
   }

   fn cause(&self) -> Option<&(dyn error::Error + 'static)> {
      match *self {
         Self::DependencyFailure(..) | Self::DependencyRejection(..) => None,
         Self::ProtobufDecode(_, ref error) => Some(error),
         Self::Request(_, ref error) | Self::Response(_, ref error) => Some(error),
         Self::Socket(ref error) => Some(error),
      }
   }

   fn source(&self) -> Option<&(dyn error::Error + 'static)> {
      match *self {
         Self::DependencyFailure(..) | Self::DependencyRejection(..) => None,
         Self::ProtobufDecode(_, ref error) => Some(error),
         Self::Request(_, ref error) | Self::Response(_, ref error) => Some(error),
         Self::Socket(ref error) => Some(error),
      }
   }
}
