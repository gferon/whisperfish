macro_rules! define_model_roles {
    (RETRIEVE $obj:ident fn $fn:ident(&self) $(via $via_fn:path)*) => {{
        let field = $obj.$fn();
        $(let field = $via_fn(field);)*
        field.into()
    }};
    (RETRIEVE $obj:ident $($field:ident).+ $(via $via_fn:path)*) => {{
        let field = $obj.$($field).+.clone();
        $(let field = $via_fn(field);)*
        field.into()
    }};
    ($vis:vis enum $enum_name:ident for $diesel_model:ty $([with offset $offset:literal])? {
     $($role:ident($($retrieval:tt)*): $name:expr),* $(,)?
    }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $enum_name {
            $($role),*
        }

        impl $enum_name {
            #[allow(unused_assignments)]
            #[allow(dead_code)]
            $vis fn role_names() -> std::collections::HashMap<i32, qmetaobject::QByteArray> {
                let mut hm = std::collections::HashMap::new();

                let mut i = 0;
                $(i = $offset;)?
                $(
                    hm.insert(i, $name.into());
                    i += 1;
                )*

                hm
            }

            $vis fn get(&self, obj: &$diesel_model) -> qmetaobject::QVariant {
                match self {
                    $(
                        Self::$role => define_model_roles!(RETRIEVE obj $($retrieval)*),
                    )*
                }
            }

            #[allow(unused)]
            $vis fn from(i: i32) -> Self {
                let rm = [$(Self::$role, )*];
                rm[i as usize]
            }
        }
    };
}

mod active_model;
pub mod attachment;
pub mod contact;
pub mod create_conversation;
pub mod device;
pub mod group;
pub mod messages;
pub mod reactions;
pub mod recipient;
pub mod sessions;

pub mod prompt;

pub use self::active_model::*;
pub use self::attachment::*;
pub use self::contact::*;
pub use self::create_conversation::*;
pub use self::device::*;
pub use self::group::*;
pub use self::messages::*;
pub use self::prompt::*;
pub use self::reactions::*;
pub use self::recipient::*;
pub use self::sessions::*;

use chrono::prelude::*;
use qmetaobject::prelude::*;

fn qdate_from_chrono<T: TimeZone>(dt: DateTime<T>) -> QDate {
    let dt = dt.with_timezone(&Local).naive_local();
    QDate::from_y_m_d(dt.year(), dt.month() as i32, dt.day() as i32)
}

fn qdatetime_from_chrono<T: TimeZone>(dt: DateTime<T>) -> QDateTime {
    let dt = dt.with_timezone(&Local).naive_local();
    let date = QDate::from_y_m_d(dt.year(), dt.month() as i32, dt.day() as i32);
    let time = QTime::from_h_m_s_ms(
        dt.hour() as i32,
        dt.minute() as i32,
        Some(dt.second() as i32),
        None,
    );

    QDateTime::from_date_time_local_timezone(date, time)
}

fn qdatetime_from_naive_option(timestamp: Option<NaiveDateTime>) -> qmetaobject::QVariant {
    timestamp
        .map(qdatetime_from_naive)
        .map(QVariant::from)
        .unwrap_or_default()
}

fn qdatetime_from_naive(timestamp: NaiveDateTime) -> QDateTime {
    // Naive in model is Utc, naive displayed should be Local
    qdatetime_from_chrono(DateTime::<Utc>::from_utc(timestamp, Utc))
}

fn qstring_from_optional_to_string(opt: Option<impl ToString>) -> QVariant {
    match opt {
        Some(s) => QString::from(s.to_string()).into(),
        None => QVariant::default(),
    }
}

fn qstring_from_option(opt: Option<impl AsRef<str>>) -> QVariant {
    match opt {
        Some(s) => QString::from(s.as_ref()).into(),
        None => QVariant::default(),
    }
}

fn qvariant_from_option<T>(val: Option<T>) -> QVariant
where
    T: Into<QVariant>,
{
    match val {
        Some(s) => s.into(),
        None => QVariant::default(),
    }
}
