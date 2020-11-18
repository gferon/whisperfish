use std::collections::HashMap;
use std::str::FromStr;

use crate::settings::*;
use crate::sfos::SailfishApp;

use actix::prelude::*;
use diesel::prelude::*;
use phonenumber::Mode;
use qmetaobject::*;

/// `/home/nemo/.local/share/system/Contacts/qtcontacts-sqlite/contacts.db`
///
/// Contains only the part after `share`.
const DB_PATH: &str = "system/Contacts/qtcontacts-sqlite/contacts.db";

#[derive(QObject, Default)]
pub struct ContactModel {
    base: qt_base_class!(trait QAbstractListModel),
    actor: Option<Addr<ContactActor>>,

    content: Vec<Contact>,

    format: qt_method!(fn(&self, string: QString) -> QString),
    name: qt_method!(fn(&self, source: QString) -> QString),
}

pub struct ContactActor {
    inner: QObjectBox<ContactModel>,
}

#[derive(Queryable)]
pub struct Contact {
    name: String,
    tel: String,
}

impl ContactActor {
    pub fn new(app: &mut SailfishApp) -> Self {
        let inner = QObjectBox::new(ContactModel::default());
        app.set_object_property("ContactModel".into(), inner.pinned());

        Self { inner }
    }
}

impl Actor for ContactActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

define_model_roles! {
    enum ContactRoles for Contact {
        Name(name via QString::from): "name",
        Tel(tel via QString::from):   "tel"
    }
}

impl ContactModel {
    fn format_helper(&self, number: &str, mode: Mode) -> Option<String> {
        let settings = Settings::default();

        let country_code = settings.get_string("country_code");

        let country = match phonenumber::country::Id::from_str(&country_code) {
            Ok(country) => country,
            Err(_) => return None,
        };

        let number = match phonenumber::parse(Some(country), number) {
            Ok(number) => number,
            Err(_) => return None,
        };

        if !phonenumber::is_valid(&number) {
            return None;
        }

        Some(number.format().mode(mode).to_string())
    }

    // The default formatter expected by QML
    fn format(&self, string: QString) -> QString {
        let string = string.to_string();
        let string = string.trim();
        if string.is_empty() {
            return QString::from("");
        }

        let string_with_plus = format!("+{}", string);

        if let Some(number) = self.format_helper(string, Mode::E164) {
            QString::from(number)
        } else if string.starts_with('+') {
            QString::from("")
        } else if let Some(number) = self.format_helper(&string_with_plus, Mode::E164) {
            QString::from(number)
        } else {
            QString::from("")
        }
    }

    fn db(&self) -> SqliteConnection {
        let path = dirs::data_local_dir().expect("find data directory");
        SqliteConnection::establish(path.join(DB_PATH).to_str().expect("UTF-8 path"))
            .expect("open contact database")
    }

    fn name(&self, source: QString) -> QString {
        use crate::schema::contacts;
        use crate::schema::phoneNumbers;

        let source = source.to_string();
        let source = source.trim();

        let conn = self.db(); // This should maybe be established only once

        // This will ensure the format to query is ok
        let e164_source = self
            .format_helper(&source, Mode::E164)
            .unwrap_or_else(|| "".into());
        let mut national_source = self
            .format_helper(&source, Mode::National)
            .unwrap_or_else(|| "".into());
        national_source.retain(|c| c != ' '); // At least FI numbers had spaces after parsing
        let source = source.to_string();

        let (name, _phone_number): (String, String) = contacts::table
            .inner_join(phoneNumbers::table)
            .select((contacts::displayLabel, phoneNumbers::phoneNumber))
            .filter(phoneNumbers::phoneNumber.like(&e164_source))
            .or_filter(phoneNumbers::phoneNumber.like(&national_source))
            .get_result(&conn)
            .unwrap_or((source.clone(), source));

        QString::from(name)
    }
}

impl QAbstractListModel for ContactModel {
    fn row_count(&self) -> i32 {
        self.content.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = ContactRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }
}
