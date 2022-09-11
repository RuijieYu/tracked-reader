use std::{
    collections::{BTreeMap, HashMap},
    fmt::{self, Display, Formatter},
    io::{self, ErrorKind, SeekFrom},
    ops::Range,
};

#[derive(Debug)]
pub struct Tracker {
    rec: Vec<TrackerEntry>,
    pos: u64,
    /// The size of the stream.  This information is only available after the
    /// first successful [`Seek::seek`] call.
    sz: Option<u64>,
}

/// A single tracker entry, containing either a successful IO operation at the
/// given inclusive range, or a failure with the error kind.
#[derive(Debug, Clone)]
pub enum TrackerEntry {
    Entry(Range<u64>),
    Error(ErrorKind),
}

/// Perform `u64 - i64`, avoiding data loss due to type casts.
const fn sub_u64_i64(lhs: u64, rhs: i64) -> u64 {
    if rhs.is_negative() {
        lhs + (-rhs) as u64
    } else {
        lhs - rhs as u64
    }
}

impl Tracker {
    /// Create an empty tracker.
    pub const fn new() -> Self {
        Self { rec: Vec::new(), pos: 0, sz: None }
    }

    /// Track a [`Read::read`] call of the inner object.
    ///
    /// [`Read::read`]: std::io::Read::read
    pub fn read(&mut self, read: io::Result<usize>) -> io::Result<usize> {
        let Self { rec: vec, pos, .. } = self;
        vec.push(match &read {
            Err(e) => TrackerEntry::Error(e.kind()),
            Ok(len) => TrackerEntry::Entry({
                let begin = *pos;
                let end = *pos + *len as u64;
                *pos = end;
                begin..end
            }),
        });
        read
    }

    /// Track a [`Seek::seek`] call of the inner object.
    ///
    /// [`Seek::seek`]: std::io::Seek::seek
    pub fn seek(
        &mut self,
        from: SeekFrom,
        seek: io::Result<u64>,
    ) -> io::Result<u64> {
        let Self { pos, sz, .. } = self;
        *pos = seek?;

        if let SeekFrom::End(e) = from {
            // sz + e = pos
            *sz = Some(sub_u64_i64(*pos, e))
        }

        Ok(*pos)
    }

    pub const fn sz(&self) -> Option<u64> {
        self.sz
    }

    pub const fn pos(&self) -> u64 {
        self.pos
    }
}

/// Collection of successful operations.  Key is the chunk index, value is a map
/// from range to number of occurences.
type OpsType = HashMap<u64, HashMap<Range<u64>, u64>>;

type ErrsType = HashMap<ErrorKind, u64>;

///
pub const DEFAULT_CHUNK: u64 = 64;

/// The report type that can report a given tracker information.
pub struct Report<const CHUNK: u64 = DEFAULT_CHUNK> {
    /// Collection of successful operations.  Key is the chunk index, value is a
    /// map from range to number of occurences.
    ops: OpsType,

    /// Collection for failed operations.  Key is the error type, value is
    /// amount of occurences.
    errs: ErrsType,

    /// ADDITIONAL data from the tracker.  This includes the current position of
    /// the cursor, and the most-recently perceived stream size.  The size may
    /// not be accurate in case the stream has since been resized.
    meta: (u64, Option<u64>),
}

impl<const CHUNK: u64> Report<CHUNK> {
    /// Create a report form an existing tracker.
    pub fn create(tracker: &Tracker) -> Self {
        let Tracker { rec, pos, sz } = tracker;
        let mut ops = HashMap::new();
        let mut errs = HashMap::new();

        for rec in rec {
            match rec {
                TrackerEntry::Error(e) => *errs.entry(*e).or_default() += 1,
                TrackerEntry::Entry(range) => Self::record_ops(&mut ops, range),
            }
        }

        Self { ops, errs, meta: (*pos, *sz) }
    }

    /// Record a single IO operation.
    fn record_ops(ops: &mut OpsType, range: &Range<u64>) {
        // start: floor-div; end: next-multiple
        let start = range.start / CHUNK * CHUNK;
        // waiting for feature int_roundings
        /* let end = range.end.next_multiple_of(CHUNK); */
        let end = ((range.end - 1) / CHUNK + 1) * CHUNK;

        (start..end).step_by(CHUNK as usize).for_each(|bucket| {
            *ops.entry(bucket).or_default().entry(range.clone()).or_default() +=
                1
        });
    }

    /// Dump into a serializable object.
    pub fn ser(&self) -> impl serde::Serialize {
        let Self { ops, errs, meta: (pos, sz) } = self;

        NativeReport {
            md: NativeReportMetadata { pos: *pos, sz: *sz },
            ops: Self::collect_ops(ops),
            errs: Self::collect_errs(errs),
        }
    }

    /// Collect successful IO information.
    fn collect_ops(ops: &OpsType) -> NativeReportOps {
        NativeReportOps(
            ops.iter().map(|(key, val)| (*key, val.values().sum())).collect(),
        )
    }

    /// Collect failed IO information.
    fn collect_errs(errs: &ErrsType) -> NativeReportErrs {
        NativeReportErrs(
            errs.iter()
                .map(|(key, val)| (ErrorKindString(key.to_string()), *val))
                .collect(),
        )
    }
}

impl<const CHUNK: u64> Display for Report<CHUNK> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&serde_yaml::to_string(&self.ser()).unwrap())
    }
}

#[derive(serde::Serialize)]
struct NativeReport {
    #[serde(rename = "metadata")]
    md: NativeReportMetadata,
    #[serde(rename = "io_operations")]
    ops: NativeReportOps,
    #[serde(rename = "io_errors")]
    errs: NativeReportErrs,
}

#[derive(serde::Serialize)]
struct NativeReportMetadata {
    #[serde(rename = "current_position")]
    pos: u64,
    #[serde(rename = "estimated_size")]
    sz: Option<u64>,
}

#[derive(serde::Serialize)]
struct NativeReportOps(BTreeMap<u64, u64>);

#[derive(Hash, PartialEq, Eq, serde::Serialize)]
struct ErrorKindString(String);
#[derive(serde::Serialize)]
struct NativeReportErrs(HashMap<ErrorKindString, u64>);
