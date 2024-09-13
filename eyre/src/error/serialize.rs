use crate::Report;
use serde::{ser::Serializer, Serialize};

impl Serialize for Report {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(format!("{:?}", self).as_ref())
    }
}
