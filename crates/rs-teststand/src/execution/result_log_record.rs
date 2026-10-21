//! One record read from a results log.

use crate::ResultLogRecordType;

/// A record returned by [`ResultLog::read_next_record`](crate::ResultLog::read_next_record).
#[derive(Debug)]
pub struct ResultLogRecord {
    /// The record type, when the engine reports one this crate names.
    pub record_type: Option<ResultLogRecordType>,
    /// The execution that produced the record.
    pub execution_id: i32,
    /// The thread that produced the record.
    pub thread_id: i32,
    /// The objects the record carries.
    pub objects: Vec<crate::PropertyObject>,
}
