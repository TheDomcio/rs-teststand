//! The kind of record `ResultLog.ReadNextRecord` returns.

/// A results-log record type (`ResultLogRecordTypes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ResultLogRecordType {
    /// Not a record type (`ResultLogRecordType_NotARecordType`).
    #[default]
    NotARecordType,
    /// Begin (`ResultLogRecordType_Begin`).
    Begin,
    /// Pre-batch (`ResultLogRecordType_PreBatch`).
    PreBatch,
    /// Batch start (`ResultLogRecordType_BatchStart`).
    BatchStart,
    /// Pre-UUT (`ResultLogRecordType_PreUUT`).
    PreUut,
    /// UUT start (`ResultLogRecordType_UUTStart`).
    UutStart,
    /// On-the-fly step results (`ResultLogRecordType_OnTheFlyStepResults`).
    OnTheFlyStepResults,
    /// UUT done (`ResultLogRecordType_UUTDone`).
    UutDone,
    /// Post-UUT (`ResultLogRecordType_PostUUT`).
    PostUut,
    /// Batch done (`ResultLogRecordType_BatchDone`).
    BatchDone,
    /// Post-batch (`ResultLogRecordType_PostBatch`).
    PostBatch,
    /// End (`ResultLogRecordType_End`).
    End,
    /// Deleted (`ResultLogRecordType_Deleted`).
    Deleted,
}

impl ResultLogRecordType {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::NotARecordType => 0,
            Self::Begin => 1,
            Self::PreBatch => 2,
            Self::BatchStart => 3,
            Self::PreUut => 4,
            Self::UutStart => 5,
            Self::OnTheFlyStepResults => 6,
            Self::UutDone => 7,
            Self::PostUut => 8,
            Self::BatchDone => 9,
            Self::PostBatch => 10,
            Self::End => 11,
            Self::Deleted => 12,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::NotARecordType,
            Self::Begin,
            Self::PreBatch,
            Self::BatchStart,
            Self::PreUut,
            Self::UutStart,
            Self::OnTheFlyStepResults,
            Self::UutDone,
            Self::PostUut,
            Self::BatchDone,
            Self::PostBatch,
            Self::End,
            Self::Deleted,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}
