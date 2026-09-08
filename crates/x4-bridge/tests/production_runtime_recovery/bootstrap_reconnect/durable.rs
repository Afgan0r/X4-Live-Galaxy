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
    if expected == 2 {
        assert_commit_loss(&connection);
    }
}

struct DurableRow {
    revision: i64,
    producer: String,
    epoch: i64,
    content: String,
    revision_digest: Vec<u8>,
    receipt_digest: Vec<u8>,
    previous: Option<i64>,
    ordinal: i64,
}

fn assert_commit_loss(connection: &Connection) {
    const EXPECTED_CONTENT_DIGEST: [u8; 32] = [
        0x06, 0x10, 0xc0, 0x84, 0x82, 0xeb, 0xf2, 0x09, 0x2b, 0xcf, 0xad, 0x7a, 0xca, 0x04,
        0x79, 0x49, 0xc7, 0xc2, 0x9f, 0xc8, 0xf4, 0xf0, 0x05, 0x2d, 0x51, 0x09, 0x20, 0x07,
        0xf4, 0x3c, 0x24, 0xe6,
    ];
    let mut statement = connection
        .prepare(
            "SELECT r.revision, r.producer_incarnation, r.transport_epoch, rr.content, r.content_digest, p.content_digest, p.previous_revision, p.ordinal FROM revisions r JOIN revision_records rr USING(section_key, revision) JOIN publication_receipts p USING(section_key, revision) ORDER BY r.revision",
        )
        .expect("durable revision query");
    let rows: Vec<DurableRow> = statement
        .query_map([], |row| {
            Ok(DurableRow {
                revision: row.get(0)?,
                producer: row.get(1)?,
                epoch: row.get(2)?,
                content: row.get(3)?,
                revision_digest: row.get(4)?,
                receipt_digest: row.get(5)?,
                previous: row.get(6)?,
                ordinal: row.get(7)?,
            })
        })
        .expect("durable rows")
        .collect::<Result<_, _>>()
        .expect("complete durable rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (rows[0].revision, rows[0].producer.as_str(), rows[0].epoch),
        (1, "producer:1", 1)
    );
    assert_eq!(
        (rows[1].revision, rows[1].producer.as_str(), rows[1].epoch),
        (2, "producer:1", 2)
    );
    assert_eq!((rows[0].previous, rows[0].ordinal), (None, 1));
    assert_eq!((rows[1].previous, rows[1].ordinal), (Some(1), 2));
    assert_eq!(rows[0].content, rows[1].content);
    for row in rows {
        assert_eq!(row.revision_digest, EXPECTED_CONTENT_DIGEST);
        assert_eq!(row.receipt_digest, EXPECTED_CONTENT_DIGEST);
    }
}
