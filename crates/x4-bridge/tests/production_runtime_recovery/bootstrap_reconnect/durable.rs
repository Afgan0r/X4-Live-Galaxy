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
    observation_version: i64,
    revision_digest: Vec<u8>,
    receipt_digest: Vec<u8>,
    previous: Option<i64>,
    ordinal: i64,
}

fn assert_commit_loss(connection: &Connection) {
    const EXPECTED_CONTENT_DIGEST: [[u8; 32]; 2] = [
        [
            0x0f, 0x83, 0x8b, 0xba, 0xa5, 0xba, 0xc3, 0x25, 0x57, 0x95, 0xc6, 0x69, 0x8a, 0x93,
            0x57, 0x7b, 0x2e, 0xfc, 0x70, 0x04, 0x8b, 0xc8, 0x14, 0x78, 0xbb, 0xf8, 0xef, 0xbf,
            0xad, 0x19, 0xd3, 0xb1,
        ],
        [
            0xf5, 0xff, 0x99, 0xe0, 0x1d, 0xa1, 0xfc, 0xb6, 0xe3, 0x54, 0x98, 0x63, 0x74, 0xf4,
            0xbf, 0x0d, 0xf9, 0x64, 0xfc, 0x52, 0xb4, 0x15, 0x68, 0xab, 0x59, 0x3e, 0x16, 0x27,
            0x42, 0xbb, 0x9b, 0xbf,
        ],
    ];
    let mut statement = connection
        .prepare(
            "SELECT r.revision, r.producer_incarnation, r.transport_epoch, rr.content, rr.observation_version, r.content_digest, p.content_digest, p.previous_revision, p.ordinal FROM revisions r JOIN revision_records rr USING(section_key, revision) JOIN publication_receipts p USING(section_key, revision) ORDER BY r.revision",
        )
        .expect("durable revision query");
    let rows: Vec<DurableRow> = statement
        .query_map([], |row| {
            Ok(DurableRow {
                revision: row.get(0)?,
                producer: row.get(1)?,
                epoch: row.get(2)?,
                content: row.get(3)?,
                observation_version: row.get(4)?,
                revision_digest: row.get(5)?,
                receipt_digest: row.get(6)?,
                previous: row.get(7)?,
                ordinal: row.get(8)?,
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
    assert_eq!(
        (rows[0].observation_version, rows[1].observation_version),
        (1, 2)
    );
    assert_eq!(rows[0].content, rows[1].content);
    for (row, expected) in rows.into_iter().zip(EXPECTED_CONTENT_DIGEST) {
        assert_eq!(row.revision_digest, expected);
        assert_eq!(row.receipt_digest, expected);
    }
}
