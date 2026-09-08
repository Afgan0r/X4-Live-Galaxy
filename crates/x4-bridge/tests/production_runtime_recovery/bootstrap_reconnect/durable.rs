use std::path::Path;

use rusqlite::Connection;

use super::DisconnectCut;

pub fn assert_state(directory: &Path, cut: DisconnectCut) {
    let connection = Connection::open(directory.join("observations.sqlite3")).expect("database");
    let expected: i64 = if matches!(cut, DisconnectCut::CommitBeforeDisposition) {
        2
    } else {
        1
    };
    for table in ["revisions", "revision_records", "publication_receipts"] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("row count");
        assert_eq!(count, expected, "unexpected {table} count for {cut:?}");
    }
    let current: (String, i64, String, i64, String, i64) = connection
        .query_row(
            "SELECT c.section_key, c.revision, r.producer_incarnation, r.transport_epoch, rr.content, p.ordinal FROM current_revisions c JOIN revisions r USING(section_key, revision) JOIN revision_records rr USING(section_key, revision) JOIN publication_receipts p USING(section_key, revision)",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("current durable state");
    assert_eq!(current.0, "carrier_b_realtime_sample");
    assert_eq!((current.1, current.3), (2, 2));
    assert_eq!(current.2, "producer:1");
    assert_eq!(
        current.4,
        "getter=GetCurRealTime\nraw_value=123.0\nsemantics=opaque_runtime_number"
    );
    assert_eq!(current.5, expected);
}
