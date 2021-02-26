#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::Migration;

use chrono::prelude::*;
use diesel::prelude::*;
use rstest::*;
use rstest_reuse::{self, *};

type MigrationList = Vec<(String, Box<dyn Migration + 'static>)>;

#[path = "migrations/orm/mod.rs"]
pub mod orm;
#[path = "migrations/schemas/mod.rs"]
pub mod schemas;

#[fixture]
fn empty_db() -> SqliteConnection {
    let conn = SqliteConnection::establish(":memory:").unwrap();
    conn.execute("PRAGMA foreign_keys = ON;").unwrap();

    conn
}

#[fixture]
fn migrations() -> MigrationList {
    let mut migrations = Vec::new();
    for subdir in std::fs::read_dir("migrations").unwrap() {
        let subdir = subdir.unwrap().path();

        if !subdir.is_dir() {
            log::warn!("Skipping non-migration {:?}", subdir);
            continue;
        }

        migrations.push((
            subdir.file_name().unwrap().to_str().unwrap().to_string(),
            diesel_migrations::migration_from(subdir).unwrap(),
        ));
    }

    migrations.sort_by_key(|f| f.0.clone());

    assert!(!migrations.is_empty());

    migrations
}

#[fixture]
fn original_go_db(empty_db: SqliteConnection) -> SqliteConnection {
    let message = r#"create table if not exists message 
            (id integer primary key, session_id integer, source text, message string, timestamp integer,
    sent integer default 0, received integer default 0, flags integer default 0, attachment text, 
            mime_type string, has_attachment integer default 0, outgoing integer default 0)"#;
    let sentq = r#"create table if not exists sentq
		(message_id integer primary key, timestamp timestamp)"#;
    let session = r#"create table if not exists session 
		(id integer primary key, source text, message string, timestamp integer,
		 sent integer default 0, received integer default 0, unread integer default 0,
         is_group integer default 0, group_members text, group_id text, group_name text,
		 has_attachment integer default 0)"#;

    diesel::sql_query(message).execute(&empty_db).unwrap();
    diesel::sql_query(sentq).execute(&empty_db).unwrap();
    diesel::sql_query(session).execute(&empty_db).unwrap();

    empty_db
}

#[fixture]
fn fixed_go_db(empty_db: SqliteConnection, mut migrations: MigrationList) -> SqliteConnection {
    drop(migrations.split_off(3));
    assert_eq!(migrations.len(), 3);
    assert_eq!(migrations[0].0, "2020-04-26-145028_0-5-message");
    assert_eq!(migrations[1].0, "2020-04-26-145033_0-5-sentq");
    assert_eq!(migrations[2].0, "2020-04-26-145036_0-5-session");

    diesel_migrations::run_migrations(
        &empty_db,
        migrations.into_iter().map(|m| m.1),
        &mut std::io::stdout(),
    )
    .unwrap();
    empty_db
}

embed_migrations!();

#[template]
#[rstest(
    db,
    case::empty_db(empty_db()),
    case::original_go_db(original_go_db(empty_db())),
    case::fixed_go_db(fixed_go_db(empty_db(), migrations()))
)]
fn initial_dbs(db: SqliteConnection) {}

#[apply(initial_dbs)]
fn run_plain_migrations(db: SqliteConnection) {
    embedded_migrations::run(&db).unwrap();
}

#[apply(initial_dbs)]
fn one_by_one(db: SqliteConnection, migrations: MigrationList) {
    for (migration_name, migration) in migrations {
        dbg!(migration_name);
        diesel_migrations::run_migrations(&db, vec![migration], &mut std::io::stdout()).unwrap();
    }

    assert!(!diesel_migrations::any_pending_migrations(&db).unwrap());
}

// As of here, we inject data in an old database, and test whether the data is still intact after
// running all the migrations.
// Insertion of the data can be done through the old models (found in `old_schemes`), and
// assertions should be done against `harbour_whisperfish::schema`.
//
// Tests usually use the following pattern:
// - a method assert_FOO(db) that puts assertions on the db in the "current" setting.
// - a bunch of `rstest`s that take different kinds of initial dbs, puts in the data and then calls
//   the migrations and the assert function.

fn assert_bunch_of_empty_sessions(db: SqliteConnection) {
    use orm::current::*;

    let session_tests = [
        |session: Session, members: Option<Vec<(GroupV1Member, Recipient)>>| {
            assert!(session.is_dm());
            assert!(members.is_none());

            let recipient = session.unwrap_dm();
            assert_eq!(recipient.e164.as_deref(), Some("+32475"));
        },
        |session: Session, members: Option<Vec<(GroupV1Member, Recipient)>>| {
            assert!(session.is_group_v1());
            let mut members = members.unwrap();
            let test = ["+32475", "+32476", "+3277"];
            members.sort_by_key(|(_, r)| r.e164.clone().unwrap());
            assert_eq!(test.len(), members.len());
            for ((_, r), t) in members.iter().zip(&test) {
                assert_eq!(r.e164.as_ref().unwrap(), t);
            }
        },
        |session: Session, members: Option<Vec<(GroupV1Member, Recipient)>>| {
            assert!(session.is_group_v1());
            let mut members = members.unwrap();
            let test = ["+33475", "+33476", "+3377"];
            members.sort_by_key(|(_, r)| r.e164.clone().unwrap());
            assert_eq!(test.len(), members.len());
            for ((_, r), t) in members.iter().zip(&test) {
                assert_eq!(r.e164.as_ref().unwrap(), t);
            }
        },
        |session: Session, members: Option<Vec<(GroupV1Member, Recipient)>>| {
            assert!(session.is_group_v1());
            let mut members = members.unwrap();
            let test = ["+32475", "+32476", "+33475", "+33476", "+3377"];
            members.sort_by_key(|(_, r)| r.e164.clone().unwrap());
            assert_eq!(test.len(), members.len());
            for ((_, r), t) in members.iter().zip(&test) {
                assert_eq!(r.e164.as_ref().unwrap(), t);
            }
        },
    ];

    let all_sessions: Vec<DbSession> = {
        use schemas::current::sessions::dsl::*;
        assert_eq!(
            session_tests.len() as i64,
            sessions.count().first::<i64>(&db).unwrap()
        );

        sessions.load(&db).unwrap()
    };

    for (session, test) in all_sessions.into_iter().zip(&session_tests) {
        dbg!(&session);

        let group = session.group_v1_id.as_ref().map(|g_id| {
            use schemas::current::group_v1s::dsl::*;
            group_v1s.filter(id.eq(g_id)).first(&db).unwrap()
        });

        let recipient = session.direct_message_recipient_id.as_ref().map(|r_id| {
            use schemas::current::recipients::dsl::*;
            recipients.filter(id.eq(r_id)).first(&db).unwrap()
        });

        let members = session.group_v1_id.as_ref().map(|g_id| {
            use schemas::current::group_v1_members::dsl::*;
            use schemas::current::recipients::dsl::recipients;
            group_v1_members
                .inner_join(recipients)
                .filter(group_v1_id.eq(g_id))
                .load(&db)
                .unwrap()
        });

        if let Some(group) = group.as_ref() {
            dbg!(group);
        }
        if let Some(recipient) = recipient.as_ref() {
            dbg!(recipient);
        }
        test(Session::from((session, recipient, group)), members);
    }
}

#[rstest]
fn bunch_of_empty_sessions(original_go_db: SqliteConnection) {
    use orm::original::*;
    use schemas::original::session::dsl::*;

    let db = original_go_db;

    let sessions = vec![
        // Just a 1-1 session
        NewSession {
            source: "+32475".into(),
            message: "Hoh.".into(),
            timestamp: NaiveDate::from_ymd(2016, 7, 9)
                .and_hms_milli(9, 10, 11, 325)
                .timestamp_millis(),
            sent: true,
            received: true,
            unread: true,
            is_group: false,
            group_members: None,
            group_id: None,
            group_name: None,
            has_attachment: false,
        },
        // A group with three members
        NewSession {
            source: "+32474".into(),
            message: "Heh.".into(),
            timestamp: NaiveDate::from_ymd(2016, 7, 8)
                .and_hms_milli(9, 10, 11, 325)
                .timestamp_millis(),
            sent: true,
            received: true,
            unread: true,
            is_group: true,
            group_members: Some("+32475,+32476,+3277".into()),
            group_id: Some("AF88".into()),
            group_name: Some("The first group".into()),
            has_attachment: false,
        },
        // Another group with distinct members
        NewSession {
            source: "".into(),
            message: "Heh.".into(),
            timestamp: NaiveDate::from_ymd(2016, 7, 8)
                .and_hms_milli(9, 10, 11, 325)
                .timestamp_millis(),
            sent: true,
            received: true,
            unread: true,
            is_group: true,
            group_members: Some("+33475,+33476,+3377".into()),
            group_id: Some("AF89".into()),
            group_name: Some("The second group".into()),
            has_attachment: false,
        },
        // Another group, now with some common members
        NewSession {
            source: "".into(),
            message: "Heh.".into(),
            timestamp: NaiveDate::from_ymd(2016, 7, 8)
                .and_hms_milli(9, 10, 11, 325)
                .timestamp_millis(),
            sent: true,
            received: true,
            unread: true,
            is_group: true,
            group_members: Some("+32475,+32476,+33475,+33476,+3377".into()),
            group_id: Some("AF90".into()),
            group_name: Some("The third group".into()),
            has_attachment: false,
        },
    ];

    let count = sessions.len();
    assert_eq!(
        diesel::insert_into(session)
            .values(sessions)
            .execute(&db)
            .unwrap(),
        count
    );

    embedded_migrations::run(&db).unwrap();
    assert_bunch_of_empty_sessions(db);
}
