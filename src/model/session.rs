#![allow(non_snake_case)]

use std::collections::HashMap;

use crate::actor;
use crate::model::*;
use crate::store::Session;

use actix::prelude::*;
use qmetaobject::*;

#[derive(QObject, Default)]
pub struct SessionModel {
    base: qt_base_class!(trait QAbstractListModel),
    pub actor: Option<Addr<actor::SessionActor>>,

    content: Vec<Session>,

    count: qt_method!(fn(&self) -> usize),
    add: qt_method!(fn(&self, id: i64, mark_read: bool)),
    remove: qt_method!(fn(&self, idx: usize)),
    removeById: qt_method!(fn(&self, id: i64)),
    reload: qt_method!(fn(&self)),

    markRead: qt_method!(fn(&self, id: usize)),
    markReceived: qt_method!(fn(&self, id: usize)),
    markSent: qt_method!(fn(&self, id: usize, message: QString)),
}

impl SessionModel {
    fn count(&self) -> usize {
        self.content.len()
    }

    /// Add or replace a Session in the model.
    fn add(&self, id: i64, mark_read: bool) {
        use futures::prelude::*;
        Arbiter::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::FetchSession { id, mark_read })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::FetchMessage({})", id);

        unimplemented!();
    }

    /// Removes session at index. This removes the session from the list model and
    /// deletes it from the database.
    fn remove(&mut self, idx: usize) {
        if idx > self.content.len() - 1 {
            log::error!("Invalid index for session model");
            return;
        }

        let sid = self.content[idx].id;

        use futures::prelude::*;
        Arbiter::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::DeleteSession { id: sid, idx })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::DeleteSession({})", idx);
    }

    /// Removes session by id. This removes the session from the list model and
    /// deletes it from the database.
    fn removeById(&self, id: i64) {
        let idx = self
            .content
            .iter()
            .position(|x| x.id == id)
            .expect("Session ID not found in session model");

        use futures::prelude::*;
        Arbiter::spawn(
            self.actor
                .as_ref()
                .unwrap()
                .send(actor::DeleteSession { id, idx })
                .map(Result::unwrap),
        );
        log::trace!("Dispatched actor::DeleteSession({})", idx);
    }

    fn reload(&self) {
        unimplemented!();
    }

    fn markRead(&self, _id: usize) {
        log::trace!("STUB: Mark read called");
        // XXX: don't forget sync messages
    }

    fn markReceived(&self, _id: usize) {
        log::trace!("STUB: Mark received called");
        // XXX: don't forget sync messages
    }

    fn markSent(&self, _id: usize, _message: QString) {
        log::trace!("STUB: Mark sent called");
        // XXX: don't forget sync messages
    }

    /// When a new message is received for a session,
    /// it gets moved up the QML by this
    pub fn set_session_first(&mut self, sess: Session) {
        (self as &mut dyn QAbstractListModel).begin_insert_rows(0, 0);
        self.content.insert(0, sess);
        (self as &mut dyn QAbstractListModel).end_insert_rows();
    }

    // Event handlers below this line

    /// Handle loaded session
    pub fn handle_sessions_loaded(&mut self, sessions: Vec<Session>) {
        // XXX: maybe this should be called before even accessing the db?
        (self as &mut dyn QAbstractListModel).begin_reset_model();
        self.content = sessions;
        (self as &mut dyn QAbstractListModel).end_reset_model();
    }

    /// Handle add-or-replace session
    pub fn handle_fetch_session(&mut self, sess: Session, mark_read: bool) {
        log::trace!("set section: session");

        let sid = sess.id;
        let mut already_unread = false;

        let found = self
            .content
            .iter()
            .enumerate()
            .find(|(_i, s)| s.id == sess.id);

        if let Some((idx, session)) = found {
            if session.unread {
                already_unread = true;
            }

            // Remove from this place so it can be added back in later
            (self as &mut dyn QAbstractListModel).begin_remove_rows(idx as i32, idx as i32);
            self.content.remove(idx);
            (self as &mut dyn QAbstractListModel).end_remove_rows();

        };

        if sess.unread && mark_read {
            use futures::prelude::*;
            Arbiter::spawn(
                self.actor
                    .as_ref()
                    .unwrap()
                    .send(actor::MarkSessionRead {sess, already_unread})
                    .map(Result::unwrap),
            );
            log::trace!("Dispatched actor::MarkSessionRead({}, {})", sid, already_unread);

            unimplemented!();
        } else if sess.unread && !already_unread {
            // let count = self.unread() + 1;

            // self.set_unread(count);
            // self.unread_changed(count);
        }

        unimplemented!();
    }

    /// When a session is marked as read and this handler called, implicitly
    /// the session will be set at the top of the QML list.
    pub fn handle_mark_session_read(&mut self, mut sess: Session, already_unread: bool) {
        sess.unread = false;

        if already_unread {
            // let count = std::cmp::min(0, self.unread() - 1);

            // self.set_unread(count);
            // self.unread_changed(count);
        }
    }

    /// Remove deleted session from QML
    pub fn handle_delete_session(&mut self, idx: usize) {
        (self as &mut dyn QAbstractListModel).begin_remove_rows(idx as i32, idx as i32);
        self.content.remove(idx);
        (self as &mut dyn QAbstractListModel).end_remove_rows();
    }
}

impl Session {
    fn section(&self) -> String {
        // XXX: stub
        "Section".into()
    }
}

define_model_roles! {
    enum SessionRoles for Session {
        ID(id):                                              "id",
        Source(source via QString::from):                    "source",
        IsGroup(is_group):                                   "isGroup",
        GroupName(group_name via qstring_from_option):       "groupName",
        GroupMembers(group_members via qstring_from_option): "groupMembers",
        Message(message via QString::from):                  "message",
        Section(fn section(&self) via QString::from):        "section",
        Timestamp(timestamp via qdatetime_from_i64):         "timestamp",
        Unread(unread):                                      "unread",
        Sent(sent):                                          "sent",
        Received(received):                                  "received",
        HasAttachment(has_attachment):                       "hasAttachment"
    }
}

impl QAbstractListModel for SessionModel {
    fn row_count(&self) -> i32 {
        self.content.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = SessionRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        SessionRoles::role_names()
    }
}
