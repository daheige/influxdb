//! Functionality for partitioning data based on a partition template.
//!
//! The partitioning template, derived partition key format, and encodings are
//! described in detail in the [`data_types::partition_template`] module.

mod bucket;
mod filter;
mod strftime;
mod traits;

use std::{borrow::Cow, num::NonZeroUsize, ops::Range};

use data_types::{
    partition_template::{
        TablePartitionTemplateOverride, TemplatePart, ENCODED_PARTITION_KEY_CHARS,
        MAXIMUM_NUMBER_OF_TEMPLATE_PARTS, PARTITION_KEY_DELIMITER, PARTITION_KEY_MAX_PART_LEN,
        PARTITION_KEY_PART_TRUNCATED, PARTITION_KEY_VALUE_EMPTY_STR, PARTITION_KEY_VALUE_NULL_STR,
    },
    PartitionKey,
};
use hashbrown::HashMap;
use mutable_batch::{MutableBatch, WritePayload};
use percent_encoding::utf8_percent_encode;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

pub use self::traits::{Batch, PartitioningColumn, TimeColumnError};
use self::{bucket::BucketHasher, strftime::StrftimeFormatter};

/// An error generating a partition key for a row.
#[allow(missing_copy_implementations)]
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum PartitionKeyError {
    /// The partition template defines a [`Template::TimeFormat`] part, but the
    /// provided strftime formatter is invalid.
    #[error("invalid strftime format in partition template")]
    InvalidStrftime,

    /// The partition template defines a [`Template::TagValue`] part, but the
    /// column type is not "tag".
    #[error("tag value partitioner does not accept input columns of type {0}")]
    TagValueNotTag(String),

    /// A "catch all" error for when a formatter returns [`std::fmt::Error`],
    /// which contains no context.
    #[error("partition key generation error")]
    FmtError(#[from] std::fmt::Error),
}

/// Returns an iterator identifying consecutive ranges for a given partition key
pub fn partition_batch<'a, T>(
    batch: &'a T,
    template: &'a TablePartitionTemplateOverride,
) -> impl Iterator<Item = (Result<String, PartitionKeyError>, Range<usize>)> + 'a
where
    T: Batch,
{
    let parts = template.len();
    if parts > MAXIMUM_NUMBER_OF_TEMPLATE_PARTS {
        panic!(
            "partition template contains {} parts, which exceeds the maximum of {} parts",
            parts, MAXIMUM_NUMBER_OF_TEMPLATE_PARTS
        );
    }

    range_encode(partition_keys(batch, template.parts()))
}

/// A [`TablePartitionTemplateOverride`] is made up of one of more
/// [`TemplatePart`]s that are rendered and joined together by
/// [`PARTITION_KEY_DELIMITER`] to form a single partition key.
///
/// To avoid allocating intermediate strings, and performing column lookups for
/// every row, each [`TemplatePart`] is converted to a [`Template`].
///
/// [`Template::fmt_row`] can then be used to render the template for that
/// particular row to the provided string, without performing any additional
/// column lookups
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum Template<'a, T: PartitioningColumn> {
    TagValue(&'a T, Option<&'a T::TagIdentityKey>),
    TimeFormat(&'a [i64], StrftimeFormatter<'a>),
    Bucket(&'a T, BucketHasher, Option<&'a T::TagIdentityKey>),

    /// This batch is missing a partitioning tag column.
    MissingTag,
}

impl<'a, T> Template<'a, T>
where
    T: PartitioningColumn,
{
    /// Renders this template to `out` for the row `idx`.
    fn fmt_row<W: std::fmt::Write>(
        &mut self,
        out: &mut W,
        idx: usize,
    ) -> Result<(), PartitionKeyError> {
        match self {
            Template::TagValue(col, last_key) if col.is_valid(idx) => {
                let this_key = col
                    .get_tag_identity_key(idx)
                    .ok_or_else(|| PartitionKeyError::TagValueNotTag(col.type_description()))?;

                // Update the "is identical" tracking key for this new,
                // potentially different key.
                *last_key = Some(this_key);

                out.write_str(encode_key_part(col.get_tag_value(this_key).unwrap()).as_ref())?
            }
            Template::TimeFormat(t, fmt) => fmt.render(t[idx], out)?,
            Template::Bucket(col, bucketer, last_key) if col.is_valid(idx) => {
                let this_key = col
                    .get_tag_identity_key(idx)
                    .ok_or_else(|| PartitionKeyError::TagValueNotTag(col.type_description()))?;
                let this_value = col.get_tag_value(this_key).unwrap();
                let bucket = bucketer.assign_bucket(this_value);

                // Update the "is identical" tracking key for this new,
                // potentially different key.
                *last_key = Some(this_key);

                write!(out, "{bucket}")?
            }
            // Either a tag that has no value for this given row index, or the
            // batch does not contain this tag at all.
            Template::TagValue(_, last_key) => {
                // This row doesn't have a tag value, which should be carried
                // forwards to be checked against the next row.
                *last_key = None;
                out.write_str(PARTITION_KEY_VALUE_NULL_STR)?
            }
            // Either a tag that has no value for this given row index, or the
            // batch does not contain this tag at all.
            Template::Bucket(_, _, last_key) => {
                // This row doesn't have a tag value, which should be carried
                // forwards to be checked against the next row.
                *last_key = None;
                out.write_str(PARTITION_KEY_VALUE_NULL_STR)?
            }
            Template::MissingTag => out.write_str(PARTITION_KEY_VALUE_NULL_STR)?,
        }

        Ok(())
    }

    /// Returns true if the partition key generated by `self` for `idx` will be
    /// identical to the last generated key.
    fn is_identical(&mut self, idx: usize) -> bool {
        match self {
            Template::TagValue(col, last_key) if col.is_valid(idx) => {
                let this_key = match col.get_tag_identity_key(idx) {
                    Some(key) => key,
                    // This is an error, but for the purposes of identical checks,
                    // it is treated as not identical, causing the error to be
                    // raised when formatting is attempted.
                    None => return false,
                };

                // Check if the key matches the last key, indicating the same value is going to
                // be rendered.
                last_key.map(|v| v == this_key).unwrap_or_default()
            }
            Template::TimeFormat(t, fmt) => {
                // Check if the last value matches the current value, after
                // optionally applying the precision reduction optimisation.
                fmt.equals_last(t[idx])
            }
            Template::Bucket(col, fmt, last_key) if col.is_valid(idx) => {
                // To perform an equality check for `idx` when it is a
                // `Bucket` template part we must check in order:
                //
                //     1. If this dictionary key is the same as the
                //        previous
                //     2. If the assigned bucket is the same as the
                //        previous
                //
                // While just checking the bucket is correct, checking
                // the dictionary key first avoids unnecessary throwaway
                // hashing work.
                let this_key = match col.get_tag_identity_key(idx) {
                    Some(key) => key,
                    // This is an error, but for the purposes of identical checks,
                    // it is treated as not identical, causing the error to be
                    // raised when formatting is attempted.
                    None => return false,
                };

                match last_key {
                    Some(v) if this_key == *v => true,
                    Some(_) => {
                        col.get_tag_value(this_key)
                            .map(|this_value| {
                                // Grab the last assigned bucket, assign
                                // a bucket for the current value and
                                // check for equality.
                                fmt.last_assigned_bucket()
                                    .map(|last_bucket| last_bucket == fmt.assign_bucket(this_value))
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default()
                    }
                    None => false,
                }
            }
            // The last row did not contain this key, and neither does this.
            Template::TagValue(_, None) | Template::Bucket(_, _, None) => true,
            // The last row did contain a key, but this one does not (therefore
            // it differs).
            Template::TagValue(_, Some(_)) | Template::Bucket(_, _, Some(_)) => false,
            // The batch does not contain this tag at all - it always matches
            // with the previous row.
            Template::MissingTag => true,
        }
    }
}

fn encode_key_part(s: &str) -> Cow<'_, str> {
    // Encode reserved characters and non-ascii characters.
    let as_str: Cow<'_, str> = utf8_percent_encode(s, &ENCODED_PARTITION_KEY_CHARS).into();

    match as_str.len() {
        0 => Cow::Borrowed(PARTITION_KEY_VALUE_EMPTY_STR),
        1..=PARTITION_KEY_MAX_PART_LEN => as_str,
        _ => {
            // This string exceeds the maximum byte length limit and must be
            // truncated.
            //
            // Truncation of unicode strings can be tricky - this implementation
            // avoids splitting unicode code-points nor graphemes. See the
            // partition_template module docs in data_types before altering
            // this.

            // Preallocate the string to hold the long partition key part.
            let mut buf = String::with_capacity(PARTITION_KEY_MAX_PART_LEN);

            // This is a slow path, re-encoding the original input string -
            // fortunately this is an uncommon path.
            //
            // Walk the string, encoding each grapheme (which includes spaces)
            // individually, tracking the total length of the encoded string.
            // Once it hits 199 bytes, stop and append a #.

            let mut bytes = 0;
            s.graphemes(true)
                .map(|v| Cow::from(utf8_percent_encode(v, &ENCODED_PARTITION_KEY_CHARS)))
                .take_while(|v| {
                    bytes += v.len(); // Byte length of encoded grapheme
                    bytes < PARTITION_KEY_MAX_PART_LEN
                })
                .for_each(|v| buf.push_str(v.as_ref()));

            // Append the truncation marker.
            buf.push(PARTITION_KEY_PART_TRUNCATED);

            assert!(buf.len() <= PARTITION_KEY_MAX_PART_LEN);

            Cow::Owned(buf)
        }
    }
}

/// Returns an iterator of partition keys for the given table batch.
///
/// This function performs deduplication on returned keys; the returned iterator
/// yields [`Some`] containing the partition key string when a new key is
/// generated, and [`None`] when the generated key would equal the last key.
fn partition_keys<'a, T>(
    batch: &'a T,
    template_parts: impl Iterator<Item = TemplatePart<'a>>,
) -> impl Iterator<Item = Option<Result<String, PartitionKeyError>>> + 'a
where
    T: Batch,
{
    // Extract the timestamp data.
    let time = batch.time_column().expect("error reading time column");

    // Convert TemplatePart into an ordered array of Template
    let mut template = template_parts
        .map(|v| match v {
            TemplatePart::TagValue(col_name) => batch
                .column(col_name)
                .map_or_else(|| Template::MissingTag, |v| Template::TagValue(v, None)),
            TemplatePart::TimeFormat(fmt) => {
                Template::TimeFormat(time, StrftimeFormatter::new(fmt))
            }
            TemplatePart::Bucket(col_name, num_buckets) => batch.column(col_name).map_or_else(
                || Template::MissingTag,
                |v| Template::Bucket(v, BucketHasher::new(num_buckets), None),
            ),
        })
        .collect::<Vec<_>>();

    // Track the length of the last yielded partition key, and pre-allocate the
    // next partition key string to match it.
    //
    // In the happy path, keys of consistent sizes are generated and the
    // allocations reach a minimum. If the keys are inconsistent, at best a
    // subset of allocations are eliminated, and at worst, a few bytes of memory
    // is temporarily allocated until the resulting string is shrunk down.
    let mut last_len = 5;

    // The first row in a batch must always be evaluated to produce a key.
    //
    // Row 0 is guaranteed to exist, otherwise attempting to read the time
    // column above would have caused a panic (no rows -> no time column).
    let first = std::iter::once(Some(evaluate_template(&mut template, &mut last_len, 0)));

    // The subsequent rows in a batch may generate the same key, and therefore a
    // dedupe check is used before allocating & populating the partition key.
    let rest = (1..batch.num_rows()).map(move |idx| {
        // Check if this partition key is going to be different from the
        // last, short-circuiting the check if it is.
        if template.iter_mut().all(|t| t.is_identical(idx)) {
            return None;
        }

        Some(evaluate_template(&mut template, &mut last_len, idx))
    });

    first.chain(rest)
}

/// Evaluate the partition template against the row indexed by `idx`.
///
/// # Panics
///
/// This method panics if `idx` exceeds the number of rows in the batch.
fn evaluate_template<T: PartitioningColumn>(
    template: &mut [Template<'_, T>],
    last_len: &mut usize,
    idx: usize,
) -> Result<String, PartitionKeyError> {
    let mut buf = String::with_capacity(*last_len);
    let template_len = template.len();

    // Evaluate each template part for this row
    for (col_idx, col) in template.iter_mut().enumerate() {
        // Evaluate the formatter for this template part against the row.
        col.fmt_row(&mut buf, idx)?;

        // If this isn't the last element in the template, insert a field
        // delimiter.
        if col_idx + 1 != template_len {
            buf.push(PARTITION_KEY_DELIMITER);
        }
    }

    *last_len = buf.len();
    Ok(buf)
}

/// Takes an iterator of [`Option`] and merges identical consecutive elements
/// together.
///
/// Any [`None`] yielded by `iterator` is added to the range for the previous
/// [`Some`].
fn range_encode<I, T>(mut iterator: I) -> impl Iterator<Item = (T, Range<usize>)>
where
    I: Iterator<Item = Option<T>>,
    T: Eq,
{
    let mut last: Option<I::Item> = None;
    let mut range: Range<usize> = 0..0;
    std::iter::from_fn(move || loop {
        match (iterator.next(), last.take()) {
            // The iterator yeilds a NULL/identical value and there is a prior value
            (Some(None), Some(v)) => {
                range.end += 1;
                last = Some(v);
            }
            // The iterator yeilds a value, and the last value matches
            (Some(cur), Some(next)) => match cur == next {
                true => {
                    range.end += 1;
                    last = Some(next);
                }
                false => {
                    let t = range.clone();
                    range.start = range.end;
                    range.end += 1;
                    last = Some(cur);
                    return Some((next.unwrap(), t));
                }
            },
            // There is no last value
            (Some(cur), None) => {
                range.end += 1;
                last = Some(cur);
            }
            (None, Some(next)) => return Some((next.unwrap(), range.clone())),
            (None, None) => return None,
        }
    })
}

/// An error partitioning a batch.
#[allow(missing_copy_implementations)]
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum PartitionWriteError {
    /// An error deriving the partition key from the partition key template.
    #[error("{0}")]
    PartitionKey(#[from] PartitionKeyError),

    /// An error accessing the time column.
    #[error("{0}")]
    TimeColumn(#[from] TimeColumnError),
}

/// A [`MutableBatch`] with a non-zero set of row ranges to write
#[derive(Debug)]
pub struct PartitionWrite<'a> {
    batch: &'a MutableBatch,
    ranges: Vec<Range<usize>>,
    min_timestamp: i64,
    max_timestamp: i64,
    row_count: NonZeroUsize,
}

impl<'a> PartitionWrite<'a> {
    /// Create a new [`PartitionWrite`] with the entire range of the provided batch
    ///
    /// # Panic
    ///
    /// Panics if the batch has no rows
    pub fn new(batch: &'a MutableBatch) -> Result<Self, PartitionWriteError> {
        let row_count = NonZeroUsize::new(batch.rows()).unwrap();
        let time = batch.time_column()?;
        let (min_timestamp, max_timestamp) = min_max_time(time);

        // This `allow` can be removed when this issue is fixed and released:
        // <https://github.com/rust-lang/rust-clippy/issues/11086>
        #[allow(clippy::single_range_in_vec_init)]
        Ok(Self {
            batch,
            ranges: vec![0..batch.rows()],
            min_timestamp,
            max_timestamp,
            row_count,
        })
    }

    /// Returns the minimum timestamp in the write
    pub fn min_timestamp(&self) -> i64 {
        self.min_timestamp
    }

    /// Returns the maximum timestamp in the write
    pub fn max_timestamp(&self) -> i64 {
        self.max_timestamp
    }

    /// Returns the number of rows in the write
    pub fn rows(&self) -> NonZeroUsize {
        self.row_count
    }

    /// Returns a [`PartitionWrite`] containing just the rows of `Self` that pass
    /// the provided time predicate, or None if no rows
    pub fn filter(&self, predicate: impl Fn(i64) -> bool) -> Option<PartitionWrite<'a>> {
        let mut min_timestamp = i64::MAX;
        let mut max_timestamp = i64::MIN;
        let mut row_count = 0_usize;

        // Construct a predicate that lets us inspect the timestamps as they are filtered
        let inspect = |t| match predicate(t) {
            true => {
                min_timestamp = min_timestamp.min(t);
                max_timestamp = max_timestamp.max(t);
                row_count += 1;
                true
            }
            false => false,
        };

        let ranges: Vec<_> = filter::filter_time(self.batch, &self.ranges, inspect);
        let row_count = NonZeroUsize::new(row_count)?;

        Some(PartitionWrite {
            batch: self.batch,
            ranges,
            min_timestamp,
            max_timestamp,
            row_count,
        })
    }

    /// Create a collection of [`PartitionWrite`] indexed by partition key
    /// from a [`MutableBatch`] and [`TablePartitionTemplateOverride`]
    pub fn partition(
        batch: &'a MutableBatch,
        partition_template: &TablePartitionTemplateOverride,
    ) -> Result<HashMap<PartitionKey, Self>, PartitionWriteError> {
        use hashbrown::hash_map::Entry;
        let time = batch.time_column()?;

        let mut partition_ranges = HashMap::new();
        for (partition, range) in partition_batch(batch, partition_template) {
            let row_count = NonZeroUsize::new(range.end - range.start).unwrap();
            let (min_timestamp, max_timestamp) = min_max_time(&time[range.clone()]);

            match partition_ranges.entry(PartitionKey::from(partition?)) {
                Entry::Vacant(v) => {
                    v.insert(PartitionWrite {
                        batch,
                        ranges: vec![range],
                        min_timestamp,
                        max_timestamp,
                        row_count,
                    });
                }
                Entry::Occupied(mut o) => {
                    let pw = o.get_mut();
                    pw.min_timestamp = pw.min_timestamp.min(min_timestamp);
                    pw.max_timestamp = pw.max_timestamp.max(max_timestamp);
                    pw.row_count = NonZeroUsize::new(pw.row_count.get() + row_count.get()).unwrap();
                    pw.ranges.push(range);
                }
            }
        }
        Ok(partition_ranges)
    }
}

impl<'a> WritePayload for PartitionWrite<'a> {
    fn write_to_batch(&self, batch: &mut MutableBatch) -> mutable_batch::Result<()> {
        batch.extend_from_ranges(self.batch, &self.ranges)
    }
}

fn min_max_time(col: &[i64]) -> (i64, i64) {
    let mut min_timestamp = i64::MAX;
    let mut max_timestamp = i64::MIN;
    for t in col {
        min_timestamp = min_timestamp.min(*t);
        max_timestamp = max_timestamp.max(*t);
    }
    (min_timestamp, max_timestamp)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    use assert_matches::assert_matches;
    use chrono::{format::StrftimeItems, DateTime, Datelike, Days, TimeZone, Utc};
    use data_types::partition_template::{
        build_column_values, test_table_partition_override, ColumnValue,
    };
    use mutable_batch::{writer::Writer, MutableBatch};
    use proptest::{prelude::*, prop_compose, proptest, strategy::Strategy};
    use rand::prelude::*;
    use schema::{Projection, TIME_COLUMN_NAME};
    use test_helpers::assert_error;

    #[test]
    fn return_err_if_no_time_column() {
        let batch = MutableBatch::new();
        let table_partition_template = Default::default();
        assert_error!(
            PartitionWrite::partition(&batch, &table_partition_template),
            PartitionWriteError::TimeColumn(TimeColumnError::NotFound),
        );
    }

    fn make_rng() -> StdRng {
        let seed = rand::rngs::OsRng.next_u64();
        println!("Seed: {seed}");
        StdRng::seed_from_u64(seed)
    }

    /// Reproducer for https://github.com/influxdata/idpe/issues/17765
    #[test]
    fn test_equals_last() {
        let ts = [
            1686756903736785920, // last_eq=false, render, set last_ptr
            42,                  // last_eq=false, render, set last_ptr
            1686756903736785920, // last_eq=false, re-use, don't change last_ptr
            1686756903736785920, // last_eq=false, re-use, don't change last_ptr
            42,                  // last_eq=true (wrong), re-use
        ];

        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, ts.len());

        writer.write_time("time", ts.into_iter()).unwrap();
        writer.commit();

        let keys =
            generate_denormalised_keys(&batch, TablePartitionTemplateOverride::default().parts())
                .unwrap();

        assert_eq!(
            keys,
            &[
                "2023-06-14",
                "1970-01-01",
                "2023-06-14",
                "2023-06-14",
                "1970-01-01",
            ]
        );
    }

    /// Generates a vector of partition key strings, or an error.
    ///
    /// This function normalises the de-duplicated output of
    /// [`partition_keys()`], returning the last observed key when the dedupe
    /// [`partition_keys()`] process returns [`None`].
    fn generate_denormalised_keys<'a, 'b: 'a, T: Batch>(
        batch: &'b T,
        template_parts: impl Iterator<Item = TemplatePart<'a>>,
    ) -> Result<Vec<String>, PartitionKeyError> {
        let mut last_ret = None;
        partition_keys(batch, template_parts)
            .map(|v| match v {
                Some(this) => {
                    last_ret = Some(this.clone());
                    this
                }
                None => last_ret
                    .as_ref()
                    .expect("must have observed prior key")
                    .clone(),
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// A fixture test asserting the default partition key format, derived from
    /// the default partition key template.
    #[test]
    fn test_default_fixture() {
        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, 1);

        writer.write_time("time", vec![1].into_iter()).unwrap();
        writer
            .write_tag("region", Some(&[0b00000001]), vec!["bananas"].into_iter())
            .unwrap();
        writer.commit();

        let template_parts =
            TablePartitionTemplateOverride::try_new(None, &Default::default()).unwrap();
        let keys: Vec<_> = partition_keys(&batch, template_parts.parts())
            .map(|v| v.expect("non-identical consecutive keys"))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(keys, vec!["1970-01-01".to_string()])
    }

    #[test]
    #[should_panic(expected = r#"error reading time column: NotFound"#)]
    fn test_zero_sized_batch() {
        let batch = MutableBatch::new();

        let template_parts = test_table_partition_override(vec![
            TemplatePart::TimeFormat("%Y-%m-%d %H:%M:%S"),
            TemplatePart::TagValue("region"),
            TemplatePart::TagValue("bananas"),
        ]);

        let keys: Vec<_> = partition_batch(&batch, &template_parts).collect::<Vec<_>>();
        assert_eq!(keys, vec![])
    }

    #[test]
    fn test_range_encode() {
        let collected: Vec<_> =
            range_encode(vec![5, 5, 5, 7, 2, 2, 3].into_iter().map(Some)).collect();
        assert_eq!(collected, vec![(5, 0..3), (7, 3..4), (2, 4..6), (3, 6..7)])
    }

    #[test]
    fn test_range_encode_sparse() {
        let collected: Vec<_> =
            range_encode(vec![Some(5), None, None, Some(7), Some(2), None, Some(3)].into_iter())
                .collect();
        assert_eq!(collected, vec![(5, 0..3), (7, 3..4), (2, 4..6), (3, 6..7)])
    }

    #[test]
    fn test_range_encode_fuzz() {
        let mut rng = make_rng();
        let original: Vec<_> = std::iter::from_fn(|| Some(rng.next_u32() % 20))
            .take(1000)
            .collect();

        let rle: Vec<_> = range_encode(original.iter().cloned().map(Some)).collect();

        let mut last_range = rle[0].1.clone();
        for (_, range) in &rle[1..] {
            assert_eq!(range.start, last_range.end);
            assert_ne!(range.start, range.end);
            last_range = range.clone();
        }

        let hydrated: Vec<_> = rle
            .iter()
            .flat_map(|(v, r)| std::iter::repeat(*v).take(r.end - r.start))
            .collect();

        assert_eq!(original, hydrated)
    }

    #[test]
    fn test_partition() {
        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, 5);

        writer
            .write_time("time", vec![1, 2, 3, 4, 5].into_iter())
            .unwrap();

        writer
            .write_tag(
                "region",
                Some(&[0b00001010]),
                vec!["west", "east"].into_iter(),
            )
            .unwrap();
        writer
            .write_tag(
                "device",
                Some(&[0b00001110]),
                vec![
                    "97c953a1-70e6-4569-80e4-59d1f49ec3fa",
                    "f1aac284-b8a1-4938-acf3-52a3d516ca14",
                    "420bb984-4d1e-48ec-bbfc-10825fbf3221",
                ]
                .into_iter(),
            )
            .unwrap();

        let template_parts = [
            TemplatePart::TimeFormat("%Y-%m-%d %H:%M:%S"),
            TemplatePart::TagValue("region"),
            TemplatePart::Bucket("device", 10),
            TemplatePart::TagValue("bananas"), // column not present
        ];

        writer.commit();

        let keys: Vec<_> = partition_keys(&batch, template_parts.clone().into_iter())
            .map(|v| v.expect("non-identical consecutive keys"))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(
            keys,
            vec![
                "1970-01-01 00:00:00|!|!|!".to_string(),
                "1970-01-01 00:00:00|west|6|!".to_string(),
                "1970-01-01 00:00:00|!|4|!".to_string(),
                "1970-01-01 00:00:00|east|5|!".to_string(),
                "1970-01-01 00:00:00|!|!|!".to_string()
            ]
        );

        let record_batch = batch.to_arrow(Projection::All).unwrap();

        let keys: Vec<_> = partition_keys(&record_batch, template_parts.into_iter())
            .map(|v| v.expect("non-identical consecutive keys"))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(
            keys,
            vec![
                "1970-01-01 00:00:00|!|!|!".to_string(),
                "1970-01-01 00:00:00|west|6|!".to_string(),
                "1970-01-01 00:00:00|!|4|!".to_string(),
                "1970-01-01 00:00:00|east|5|!".to_string(),
                "1970-01-01 00:00:00|!|!|!".to_string()
            ]
        );
    }

    #[test]
    fn test_bucket_fixture() {
        let mut bucketer = BucketHasher::new(10);
        assert_eq!(bucketer.assign_bucket("foo"), 6);
        assert_eq!(bucketer.last_assigned_bucket(), Some(6));
        assert_eq!(bucketer.assign_bucket("bat"), 5);
        assert_eq!(bucketer.last_assigned_bucket(), Some(5));
        assert_eq!(bucketer.assign_bucket("qux"), 5);
        assert_eq!(bucketer.last_assigned_bucket(), Some(5));
    }

    #[test]
    fn test_sparse_representation() {
        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, 7);

        writer
            .write_time(
                "time",
                vec![
                    1,
                    1,
                    1,
                    1,
                    1685971961464736000,
                    1685971961464736000,
                    1685971961464736000,
                ]
                .into_iter(),
            )
            .unwrap();

        writer
            .write_tag(
                "region",
                Some(&[0b01111111]),
                vec![
                    "platanos", "platanos", "platanos", "platanos", "platanos", "platanos",
                    "bananas",
                ]
                .into_iter(),
            )
            .unwrap();

        writer
            .write_tag(
                "device",
                Some(&[0b01111111]),
                vec!["foo", "bat", "qux", "bat", "foo", "foo", "foo"].into_iter(), // `bat` and `qux` both go to bucket 5, so those 3 values should yield the same key
            )
            .unwrap();

        let template_parts = [
            TemplatePart::TimeFormat("%Y-%m-%d %H:%M:%S"),
            TemplatePart::TagValue("region"),
            TemplatePart::Bucket("device", 10),
            TemplatePart::TagValue("bananas"), // column not present
        ];

        writer.commit();

        let mut iter = partition_keys(&batch, template_parts.into_iter());

        assert_eq!(
            iter.next().unwrap(),
            Some(Ok("1970-01-01 00:00:00|platanos|6|!".to_string()))
        );
        assert_eq!(
            iter.next().unwrap(),
            Some(Ok("1970-01-01 00:00:00|platanos|5|!".to_string()))
        );
        assert_eq!(iter.next().unwrap(), None);
        assert_eq!(iter.next().unwrap(), None);
        assert_eq!(
            iter.next().unwrap(),
            Some(Ok("2023-06-05 13:32:41|platanos|6|!".to_string()))
        );
        assert_eq!(iter.next().unwrap(), None);
        assert_eq!(
            iter.next().unwrap(),
            Some(Ok("2023-06-05 13:32:41|bananas|6|!".to_string()))
        );
    }

    #[test]
    fn partitioning_on_fields_panics() {
        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, 5);

        writer
            .write_time("time", vec![1, 2, 3, 4, 5].into_iter())
            .unwrap();

        writer
            .write_string(
                "region",
                Some(&[0b00001010]),
                vec!["west", "east"].into_iter(),
            )
            .unwrap();

        let template_parts = [TemplatePart::TagValue("region")];

        writer.commit();

        let got: Result<Vec<_>, _> = generate_denormalised_keys(&batch, template_parts.into_iter());
        assert_matches::assert_matches!(got, Err(PartitionKeyError::TagValueNotTag(_)));
    }

    #[test]
    fn bucketing_on_fields_panics() {
        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, 5);

        writer
            .write_time("time", vec![1, 2, 3, 4, 5].into_iter())
            .unwrap();

        writer
            .write_string(
                "region",
                Some(&[0b00001010]),
                vec!["west", "east"].into_iter(),
            )
            .unwrap();

        let template_parts = [TemplatePart::Bucket("region", 10)];

        writer.commit();

        let got: Result<Vec<_>, _> = generate_denormalised_keys(&batch, template_parts.into_iter());
        assert_matches::assert_matches!(got, Err(PartitionKeyError::TagValueNotTag(_)));
    }

    fn identity<'a, T>(s: T) -> ColumnValue<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        ColumnValue::Identity(s.into())
    }

    fn prefix<'a, T>(s: T) -> ColumnValue<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        ColumnValue::Prefix(s.into())
    }

    fn year(y: i32) -> ColumnValue<'static> {
        ColumnValue::Datetime {
            begin: Utc.with_ymd_and_hms(y, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(y + 1, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    fn bucket(bucket_id: u32) -> ColumnValue<'static> {
        ColumnValue::Bucket(bucket_id)
    }

    // Generate a test that asserts the derived partition key matches
    // "want_key", when using the provided "template" parts and set of "tags".
    //
    // Additionally validates that the derived key is reversible into the
    // expected set of "want_reversed_tags" from the original inputs.
    macro_rules! test_partition_key {
        (
            $name:ident,
            template = $template:expr,              // Array/vec of TemplatePart
            tags = $tags:expr,                      // Array/vec of (tag_name, value) tuples
            want_key = $want_key:expr,              // Expected partition key string
            want_reversed_tags = $want_reversed_tags:expr // Array/vec of (tag_name, value) reversed from $tags
        ) => {
            paste::paste! {
                #[test]
                fn [<test_partition_key_ $name>]() {
                    let mut batch = MutableBatch::new();
                    let mut writer = Writer::new(&mut batch, 1);

                    let template = $template.into_iter().collect::<Vec<_>>();
                    let template = test_table_partition_override(template);

                    // Timestamp: 2023-05-29T13:03:16Z
                    writer
                        .write_time("time", vec![1685365396931384064].into_iter())
                        .unwrap();

                    for (col, value) in $tags {
                        let v = String::from(value);
                        writer
                            .write_tag(col, Some(&[0b00000001]), vec![v.as_str()].into_iter())
                            .unwrap();
                    }

                    writer.commit();

                    // Generate the full set of partition keys, inserting the
                    // last observed value when the next key is identical to
                    // normalise the values.
                    let keys = generate_denormalised_keys(&batch, template.parts())
                        .unwrap();
                    assert_eq!(keys, vec![$want_key.to_string()], "generated key differs");

                    // Reverse the encoding.
                    let reversed = build_column_values(&template, &keys[0]);

                    // Expect the tags to be (str, ColumnValue) for the
                    // comparison
                    let want: Vec<(&str, ColumnValue<'_>)> = $want_reversed_tags
                        .into_iter()
                        .collect();

                    let got = reversed.collect::<Vec<_>>();
                    assert_eq!(got, want, "reversed key differs");
                }
            }
        };
    }

    test_partition_key!(
        simple,
        template = [
            TemplatePart::TimeFormat("%Y"),
            TemplatePart::TagValue("a"),
            TemplatePart::TagValue("b"),
            TemplatePart::Bucket("c", 5),
        ],
        tags = [
            ("a", "bananas"),
            ("b", "are_good"),
            ("c", "for_test_strings")
        ],
        want_key = "2023|bananas|are_good|1",
        want_reversed_tags = [
            (TIME_COLUMN_NAME, year(2023)),
            ("a", identity("bananas")),
            ("b", identity("are_good")),
            ("c", bucket(1)),
        ]
    );

    test_partition_key!(
        non_ascii,
        template = [
            TemplatePart::TimeFormat("%Y"),
            TemplatePart::TagValue("a"),
            TemplatePart::TagValue("b"),
        ],
        tags = [("a", "bananas"), ("b", "plátanos")],
        want_key = "2023|bananas|pl%C3%A1tanos",
        want_reversed_tags = [
            (TIME_COLUMN_NAME, year(2023)),
            ("a", identity("bananas")),
            ("b", identity("plátanos")),
        ]
    );

    test_partition_key!(
        single_tag_template_tag_not_present,
        template = [TemplatePart::TagValue("a")],
        tags = [("b", "bananas")],
        want_key = "!",
        want_reversed_tags = []
    );

    test_partition_key!(
        single_bucket_template_tag_not_present,
        template = [TemplatePart::Bucket("a", 10)],
        tags = [("b", "bananas")],
        want_key = "!",
        want_reversed_tags = []
    );

    test_partition_key!(
        single_tag_template_tag_empty,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", "")],
        want_key = "^",
        want_reversed_tags = [("a", identity(""))]
    );

    test_partition_key!(
        single_bucket_template_tag_empty,
        template = [TemplatePart::Bucket("a", 10)],
        tags = [("a", "")],
        want_key = "0",
        want_reversed_tags = [("a", bucket(0))]
    );

    test_partition_key!(
        missing_tag,
        template = [
            TemplatePart::TagValue("a"),
            TemplatePart::TagValue("b"),
            TemplatePart::Bucket("c", 10)
        ],
        tags = [("a", "bananas")],
        want_key = "bananas|!|!",
        want_reversed_tags = [("a", identity("bananas"))]
    );

    test_partition_key!(
        unambiguous,
        template = [
            TemplatePart::TimeFormat("%Y"),
            TemplatePart::TagValue("a"),
            TemplatePart::TagValue("b"),
            TemplatePart::TagValue("c"),
            TemplatePart::TagValue("d"),
            TemplatePart::TagValue("e"),
        ],
        tags = [("a", "|"), ("b", "!"), ("d", "%7C%21%257C"), ("e", "^")],
        want_key = "2023|%7C|%21|!|%257C%2521%25257C|%5E",
        want_reversed_tags = [
            (TIME_COLUMN_NAME, year(2023)),
            ("a", identity("|")),
            ("b", identity("!")),
            ("d", identity("%7C%21%257C")),
            ("e", identity("^"))
        ]
    );

    test_partition_key!(
        truncated_char_reserved,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", "#")],
        want_key = "%23",
        want_reversed_tags = [("a", identity("#"))]
    );

    // Keys < 200 bytes long should not be truncated.
    test_partition_key!(
        truncate_length_199,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", "A".repeat(199))],
        want_key = "A".repeat(199),
        want_reversed_tags = [("a", identity("A".repeat(199)))]
    );

    // Keys of exactly 200 bytes long should not be truncated.
    test_partition_key!(
        truncate_length_200,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", "A".repeat(200))],
        want_key = "A".repeat(200),
        want_reversed_tags = [("a", identity("A".repeat(200)))]
    );

    // Keys > 200 bytes long should be truncated to exactly 200 bytes,
    // terminated by a # character.
    test_partition_key!(
        truncate_length_201,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", "A".repeat(201))],
        want_key = format!("{}#", "A".repeat(199)),
        want_reversed_tags = [("a", prefix("A".repeat(199)))]
    );

    // A key ending in an encoded sequence that does not cross the cut-off point
    // is preserved.
    //
    // This subtest generates a key of:
    //
    //      `A..<repeats>%`
    //                      ^ cutoff
    //
    // Which when encoded, becomes:
    //
    //      `A..<repeats>%25`
    //                      ^ cutoff
    //
    // So the entire encoded sequence should be preserved.
    test_partition_key!(
        truncate_encoding_sequence_ok,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}%", "A".repeat(197)))],
        want_key = format!("{}%25", "A".repeat(197)), // Not truncated
        want_reversed_tags = [("a", identity(format!("{}%", "A".repeat(197))))]
    );

    // A key ending in an encoded sequence should not be split.
    //
    // This subtest generates a key of:
    //
    //      `A..<repeats>%`
    //                    ^ cutoff
    //
    // Which when encoded, becomes:
    //
    //      `A..<repeats>% 25`            (space added for clarity)
    //                    ^ cutoff
    //
    // Where naive slicing would result in truncating an encoding sequence and
    // therefore the whole encoded sequence should be truncated.
    test_partition_key!(
        truncate_encoding_sequence_truncated_1,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}%", "A".repeat(198)))],
        want_key = format!("{}#", "A".repeat(198)), // Truncated
        want_reversed_tags = [("a", prefix("A".repeat(198)))]
    );

    // A key ending in an encoded sequence should not be split.
    //
    // This subtest generates a key of:
    //
    //      `A..<repeats>%`
    //                     ^ cutoff
    //
    // Which when encoded, becomes:
    //
    //      `A..<repeats>%2 5`            (space added for clarity)
    //                     ^ cutoff
    //
    // Where naive slicing would result in truncating an encoding sequence and
    // therefore the whole encoded sequence should be truncated.
    test_partition_key!(
        truncate_encoding_sequence_truncated_2,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}%", "A".repeat(199)))],
        want_key = format!("{}#", "A".repeat(199)), // Truncated
        want_reversed_tags = [("a", prefix("A".repeat(199)))]
    );

    // A key ending in a unicode code-point should never be split.
    //
    // This subtest generates a key of:
    //
    //      `A..<repeats>🍌`
    //                         ^ cutoff
    //
    // Which when encoded, becomes:
    //
    //      `A..<repeats>%F0%9F%8D%8C`
    //                         ^ cutoff
    //
    // Therefore the entire code-point should be removed from the truncated
    // output.
    //
    // This test MUST NOT fail, or an invalid UTF-8 string is being generated
    // which is unusable in languages (like Rust).
    //
    // Advances the cut-off to ensure the position within the code-point doesn't
    // affect the output.
    test_partition_key!(
        truncate_within_code_point_1,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}🍌", "A".repeat(194)))],
        want_key = format!("{}#", "A".repeat(194)),
        want_reversed_tags = [("a", prefix("A".repeat(194)))]
    );
    test_partition_key!(
        truncate_within_code_point_2,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}🍌", "A".repeat(195)))],
        want_key = format!("{}#", "A".repeat(195)),
        want_reversed_tags = [("a", prefix("A".repeat(195)))]
    );
    test_partition_key!(
        truncate_within_code_point_3,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}🍌", "A".repeat(196)))],
        want_key = format!("{}#", "A".repeat(196)),
        want_reversed_tags = [("a", prefix("A".repeat(196)))]
    );

    // A key ending in a unicode grapheme should never be split.
    //
    // This subtest generates a key of:
    //
    //      `A..<repeats>நிbananas`
    //                   ^ cutoff
    //
    // Which when encoded, becomes:
    //
    //      `A..<repeats>நிbananas`    (within a grapheme)
    //                   ^ cutoff
    //
    // Therefore the entire grapheme (நி) should be removed from the truncated
    // output.
    //
    // This is a conservative implementation, and may be relaxed in the future.
    //
    // This first test asserts that a grapheme can be included, and then
    // subsequent tests increment the cut-off point by 1 byte each time.
    test_partition_key!(
        truncate_within_grapheme_0,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(181)))],
        want_key = format!("{}%E0%AE%A8%E0%AE%BF#", "A".repeat(181)),
        want_reversed_tags = [("a", prefix(format!("{}நி", "A".repeat(181))))]
    );
    test_partition_key!(
        truncate_within_grapheme_1,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(182)))],
        want_key = format!("{}#", "A".repeat(182)),
        want_reversed_tags = [("a", prefix("A".repeat(182)))]
    );
    test_partition_key!(
        truncate_within_grapheme_2,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(183)))],
        want_key = format!("{}#", "A".repeat(183)),
        want_reversed_tags = [("a", prefix("A".repeat(183)))]
    );
    test_partition_key!(
        truncate_within_grapheme_3,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(184)))],
        want_key = format!("{}#", "A".repeat(184)),
        want_reversed_tags = [("a", prefix("A".repeat(184)))]
    );
    test_partition_key!(
        truncate_within_grapheme_4,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(185)))],
        want_key = format!("{}#", "A".repeat(185)),
        want_reversed_tags = [("a", prefix("A".repeat(185)))]
    );
    test_partition_key!(
        truncate_within_grapheme_5,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(186)))],
        want_key = format!("{}#", "A".repeat(186)),
        want_reversed_tags = [("a", prefix("A".repeat(186)))]
    );
    test_partition_key!(
        truncate_within_grapheme_6,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(187)))],
        want_key = format!("{}#", "A".repeat(187)),
        want_reversed_tags = [("a", prefix("A".repeat(187)))]
    );
    test_partition_key!(
        truncate_within_grapheme_7,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(188)))],
        want_key = format!("{}#", "A".repeat(188)),
        want_reversed_tags = [("a", prefix("A".repeat(188)))]
    );
    test_partition_key!(
        truncate_within_grapheme_8,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(189)))],
        want_key = format!("{}#", "A".repeat(189)),
        want_reversed_tags = [("a", prefix("A".repeat(189)))]
    );
    test_partition_key!(
        truncate_within_grapheme_9,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நிbananas", "A".repeat(190)))],
        want_key = format!("{}#", "A".repeat(190)),
        want_reversed_tags = [("a", prefix("A".repeat(190)))]
    );

    // As above, but the grapheme is the last portion of the generated string
    // (no trailing bananas).
    test_partition_key!(
        truncate_grapheme_identity,
        template = [TemplatePart::TagValue("a")],
        tags = [("a", format!("{}நி", "A".repeat(182)))],
        want_key = format!("{}%E0%AE%A8%E0%AE%BF", "A".repeat(182)),
        want_reversed_tags = [("a", identity(format!("{}நி", "A".repeat(182))))]
    );

    /// A test using an invalid strftime format string.
    #[test]
    fn test_invalid_strftime() {
        let mut batch = MutableBatch::new();
        let mut writer = Writer::new(&mut batch, 1);

        writer.write_time("time", vec![1].into_iter()).unwrap();
        writer
            .write_tag("region", Some(&[0b00000001]), vec!["bananas"].into_iter())
            .unwrap();
        writer.commit();

        let template = [TemplatePart::TimeFormat("%3F")]
            .into_iter()
            .collect::<Vec<_>>();
        let template = test_table_partition_override(template);

        let ret = partition_keys(&batch, template.parts())
            .map(|v| v.expect("non-identical consecutive keys"))
            .collect::<Result<Vec<_>, _>>();

        assert_matches!(ret, Err(PartitionKeyError::InvalidStrftime));
    }

    #[test]
    #[should_panic(
        expected = "partition template contains 9 parts, which exceeds the maximum of 8 parts"
    )]
    fn test_too_many_parts() {
        let template = test_table_partition_override(
            std::iter::repeat(TemplatePart::TagValue("bananas"))
                .take(9)
                .collect(),
        );

        let _ = partition_batch(&MutableBatch::new(), &template);
    }

    // These values are arbitrarily chosen when building an input to the
    // partitioner.

    // Arbitrary tag names are selected from this set of candidates (to ensure
    // there's always some overlap, rather than truly random strings).
    const TEST_TAG_NAME_SET: &[&str] = &["A", "B", "C", "D", "E", "F"];

    // Arbitrary template parts are selected from this set.
    const TEST_TEMPLATE_PARTS: &[TemplatePart<'static>] = &[
        TemplatePart::TimeFormat("%Y|%m|%d!-string"),
        TemplatePart::TimeFormat("%Y|%m|%d!-%%bananas"),
        TemplatePart::TimeFormat("%Y/%m/%d"),
        TemplatePart::TimeFormat("%Y-%m-%d"),
        TemplatePart::TagValue(""),
        TemplatePart::TagValue("A"),
        TemplatePart::TagValue("B"),
        TemplatePart::TagValue("C"),
        TemplatePart::TagValue("tags!"),
        TemplatePart::TagValue("%tags!"),
        TemplatePart::TagValue("my_tag"),
        TemplatePart::TagValue("my|tag"),
        TemplatePart::TagValue("%%%%|!!!!|"),
        TemplatePart::Bucket("D", 10),
        TemplatePart::Bucket("E", 100),
        TemplatePart::Bucket("F", 1000),
    ];

    prop_compose! {
        /// Yields a vector of up to [`MAXIMUM_NUMBER_OF_TEMPLATE_PARTS`] unique
        /// template parts, chosen from [`TEST_TEMPLATE_PARTS`].
        fn arbitrary_template_parts()(set in proptest::collection::vec(
                proptest::sample::select(TEST_TEMPLATE_PARTS),
                (1, MAXIMUM_NUMBER_OF_TEMPLATE_PARTS) // Set size range
            )) -> Vec<TemplatePart<'static>> {
            let mut set = set;
            set.dedup_by(|a, b| format!("{a:?}") == format!("{b:?}"));
            set
        }
    }

    prop_compose! {
        /// Yield a HashMap of between 1 and 10 (column_name, random string
        /// value) with tag names chosen from [`TEST_TAG_NAME_SET`].
        fn arbitrary_tag_value_map()(v in proptest::collection::hash_map(
                proptest::sample::select(TEST_TAG_NAME_SET).prop_map(ToString::to_string),
                any::<String>(),
                (1, 10) // Set size range
            )) -> HashMap<String, String> {
            v
        }
    }

    prop_compose! {
        /// Yield a Vec containing an identical timestamp run of random length,
        /// up to `max_run_len`,
        fn arbitrary_timestamp_run(max_run_len: usize)(v in 0_i64..i64::MAX, run_len in 1..max_run_len) -> Vec<i64> {
            let mut x = Vec::with_capacity(run_len);
            x.resize(run_len, v);
            x
        }
    }

    /// Yield a Vec of timestamp values that more accurately model real
    /// timestamps than pure random selection.
    ///
    /// Runs of identical timestamps are generated with
    /// [`arbitrary_timestamp_run()`], which are then shuffled to produce a list
    /// of timestamps with limited repeats, sometimes consecutively.
    fn arbitrary_timestamps() -> impl Strategy<Value = Vec<i64>> {
        proptest::collection::vec(arbitrary_timestamp_run(6), 10..100)
            .prop_map(|v| v.into_iter().flatten().collect::<Vec<_>>())
            .prop_shuffle()
    }

    enum ExpectedColumnValue {
        String(String),
        TSRange(DateTime<Utc>, DateTime<Utc>),
        Bucket(u32),
    }

    impl ExpectedColumnValue {
        fn expect_string(&self) -> &String {
            match self {
                Self::String(s) => s,
                Self::TSRange(_, _) => panic!("expected string, got TS range"),
                Self::Bucket(_) => panic!("expected string, got bucket id"),
            }
        }

        fn expect_ts_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
            match self {
                Self::String(_) => panic!("expected TS range, got string"),
                Self::TSRange(b, e) => (*b, *e),
                Self::Bucket(_) => panic!("expected TS range, got bucket id"),
            }
        }

        fn expect_bucket_id(&self) -> u32 {
            match self {
                Self::String(_) => panic!("expected bucket id, got string"),
                Self::TSRange(_, _) => panic!("expected bucket id, got TS range"),
                Self::Bucket(bucket_id) => *bucket_id,
            }
        }
    }

    proptest! {
        /// A property test that asserts a write comprised of an arbitrary
        /// subset of [`TEST_TAG_NAME_SET`] with randomised values, that is
        /// partitioned using a partitioning template arbitrarily selected from
        /// [`TEST_TEMPLATE_PARTS`], can be reversed to the full set of tags
        /// and/or hash-bucket IDs via [`build_column_values()`].
        #[test]
        fn prop_reversible_mapping(
            template in arbitrary_template_parts(),
            tag_values in arbitrary_tag_value_map(),
            ts in 0_i64..i64::MAX,
        ) {
            let mut batch = MutableBatch::new();
            let mut writer = Writer::new(&mut batch, 1);

            let template = template.clone().into_iter().collect::<Vec<_>>();
            let template = test_table_partition_override(template);

            writer
                .write_time("time", vec![ts].into_iter())
                .unwrap();

            for (col, value) in &tag_values {
                writer
                    .write_tag(col.as_str(), Some(&[0b00000001]), vec![value.as_str()].into_iter())
                    .unwrap();
            }

            writer.commit();
            let keys: Vec<_> = generate_denormalised_keys(&batch, template.parts())
                .unwrap();
            assert_eq!(keys.len(), 1);

            // Reverse the encoding.
            let reversed: Vec<(&str, ColumnValue<'_>)> = build_column_values(&template, &keys[0]).collect();

            // Build the expected set of reversed tags by filtering out any
            // NULL tags (preserving empty string values).
            let ts = Utc.timestamp_nanos(ts);
            let want_reversed: Vec<(&str, ExpectedColumnValue)> = template.parts().filter_map(|v| match v {
                TemplatePart::TagValue(col_name) if tag_values.contains_key(col_name) => {
                    // This tag had a (potentially empty) value wrote and should
                    // appear in the reversed output.
                    Some((col_name, ExpectedColumnValue::String(tag_values.get(col_name).unwrap().to_string())))
                }
                TemplatePart::TimeFormat("%Y/%m/%d" | "%Y-%m-%d") => {
                    let begin = Utc.with_ymd_and_hms(ts.year(), ts.month(), ts.day(), 0, 0, 0).unwrap();
                    let end = begin + Days::new(1);
                    Some((TIME_COLUMN_NAME, ExpectedColumnValue::TSRange(begin, end)))
                }
                TemplatePart::Bucket(col_name, num_buckets) if tag_values.contains_key(col_name) => {
                    // Hash-bucketing is not fully-reversible from value to
                    // tag-name (intentionally so, it makes it much simpler to
                    // implement).
                    //
                    // The test must assign buckets as they are when the
                    // partition key is rendered.
                    let want_bucket = BucketHasher::new(num_buckets).assign_bucket(tag_values.get(col_name).unwrap());
                    Some((col_name, ExpectedColumnValue::Bucket(want_bucket)))
                }
                _ => None,
            }).collect();

            assert_eq!(want_reversed.len(), reversed.len());

            for ((want_col, want_val), (got_col, got_val)) in want_reversed.iter().zip(reversed.iter()) {
                assert_eq!(got_col, want_col, "column names differ");

                match got_val {
                    ColumnValue::Identity(_) => {
                        // An identity is both equal to, and a prefix of, the
                        // original value.
                        let want_val = want_val.expect_string();
                        assert_eq!(got_val, &want_val, "identity values differ");
                        assert!(
                            got_val.is_prefix_match_of(want_val),
                            "prefix mismatch; {:?} is not a prefix of {:?}",
                            got_val,
                            want_val,
                        );
                    },
                    ColumnValue::Prefix(_) => {
                        let want_val = want_val.expect_string();
                        assert!(
                            got_val.is_prefix_match_of(want_val),
                            "prefix mismatch; {:?} is not a prefix of {:?}",
                            got_val,
                            want_val,
                        );
                    },
                    ColumnValue::Datetime{..} => {
                        let (want_begin, want_end) = want_val.expect_ts_range();
                        match got_val {
                            ColumnValue::Datetime{begin, end} => {
                                assert_eq!(want_begin, *begin);
                                assert_eq!(want_end, *end);
                            }
                            _ => panic!("expected datatime column value but got: {:?}", got_val)
                        }
                    },
                    ColumnValue::Bucket(got_bucket_id) => {
                        let want_bucket_id = want_val.expect_bucket_id();
                        assert_eq!(*got_bucket_id, want_bucket_id);
                    }
                };
            }
        }

        /// A property test that asserts the partitioner tolerates (does not
        /// panic) randomised, potentially invalid strftime formatter strings.
        #[test]
        fn prop_arbitrary_strftime_format(fmt in any::<String>()) {
            let mut batch = MutableBatch::new();
            let mut writer = Writer::new(&mut batch, 1);

            // This sequence causes chrono's formatter to panic with a "do not
            // use this" message...
            //
            // This is validated to not be part of the formatter (among other
            // invalid sequences) when constructing a template from the user
            // input/proto.
            //
            // Uniquely this causes a panic, whereas others do not - so it must
            // be filtered out when fuzz-testing that invalid sequences do not
            // cause a panic in the key generator.
            prop_assume!(!fmt.contains("%#z"));

            // Generate a single time-based partitioning template with a
            // randomised format string.
            let template = vec![
                TemplatePart::TimeFormat(&fmt),
            ];
            let template = test_table_partition_override(template);

            // Timestamp: 2023-05-29T13:03:16Z
            writer
                .write_time("time", vec![1685365396931384064].into_iter())
                .unwrap();

            writer
                .write_tag("bananas", Some(&[0b00000001]), vec!["great"].into_iter())
                .unwrap();

            writer.commit();
            let ret = partition_keys(&batch, template.parts())
                .map(|v| v.expect("non-identical consecutive keys"))
                .collect::<Result<Vec<_>, _>>();

            // The is allowed to succeed or fail under this test (but not
            // panic), and the returned error/value must match certain
            // properties:
            match ret {
                Ok(v) => { assert_eq!(v.len(), 1); },
                Err(e) => { assert_matches!(e, PartitionKeyError::InvalidStrftime); },
            }
        }

        // Drives the strftime formatter through the "front door", using the
        // same interface as a user would call to partition data. This validates
        // the integration between the various formatters, range encoders,
        // dedupe, etc.
        #[test]
        fn prop_strftime_integration(
            times in arbitrary_timestamps(),
            format in prop_oneof![
                Just("%Y-%m-%d"), // Default scheme
                Just("%s")        // Unix seconds, to drive increased cache miss rate in strftime formatter
            ]
        ) {
            use std::fmt::Write;

            let mut batch = MutableBatch::new();
            let mut writer = Writer::new(&mut batch, times.len());
            let row_count = times.len();

            let template = test_table_partition_override(vec![TemplatePart::TimeFormat(format)]);

            writer
                .write_time("time", times.clone().into_iter())
                .unwrap();

            writer.commit();

            let fmt = StrftimeItems::new(format);
            let iter = partition_batch(&batch, &template);

            let mut observed_rows = 0;

            // For each partition key and the calculated row range
            for (key, range) in iter {
                let key = key.unwrap();

                observed_rows += range.len();

                // Validate all rows in that range render to the same timestamp
                // value as the partition key when using the same format, using
                // a known-good formatter.
                for ts in &times[range] {
                    // Generate the control string.
                    let mut control = String::new();
                    let _ = write!(
                        control,
                        "{}",
                        Utc.timestamp_nanos(*ts)
                            .format_with_items(fmt.clone())
                    );
                    assert_eq!(control, key);
                }
            }

            assert_eq!(observed_rows, row_count);
        }
    }
}
