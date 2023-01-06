#![allow(non_snake_case)]

use std::collections::HashMap;

use crate::store::observer::{EventObserving, Interest};
use crate::store::orm::{GroupV1Member, GroupV2Member};
use crate::store::{orm, Storage};
use crate::{model::*, schema};
use qmeta_async::with_executor;
use qmetaobject::prelude::*;

/// QML-constructable object that interacts with a single recipient.
#[derive(Default)]
pub struct GroupImpl {
    id: Option<String>,
    group_v1: Option<orm::GroupV1>,
    group_v2: Option<orm::GroupV2>,

    membership_list: QObjectBox<GroupMembershipListModel>,
}

crate::observing_model! {
    pub struct Group(GroupImpl) {
        groupId: QString; READ get_group_id WRITE set_group_id,
        isGroupV1: bool; READ get_is_group_v1,
        isGroupV2: bool; READ get_is_group_v2,

        valid: bool; READ get_valid,

        members: QVariant; READ members,
        member_count: i32; READ member_count,
    }
}

impl EventObserving for GroupImpl {
    fn observe(&mut self, storage: Storage, _event: crate::store::observer::Event) {
        if self.id.is_some() {
            self.init(storage);
        }
    }

    fn interests(&self) -> Vec<Interest> {
        let membership_list = self.membership_list.pinned();
        // XXX This should be members and new members filtered by id, instead of whole tables.
        let members = std::iter::once(Interest::whole_table(schema::group_v1_members::table))
            .chain(std::iter::once(Interest::whole_table(
                schema::group_v2_members::table,
            )))
            .chain(
                membership_list
                    .borrow()
                    .content
                    .iter()
                    .flat_map(|(_membership, recipient)| recipient.interests()),
            );
        self.group_v1
            .iter()
            .flat_map(orm::GroupV1::interests)
            .chain(self.group_v2.iter().flat_map(orm::GroupV2::interests))
            .chain(members)
            .collect()
    }
}

impl GroupImpl {
    fn get_group_id(&self) -> QString {
        self.id.clone().unwrap_or_default().into()
    }

    fn get_is_group_v1(&self) -> bool {
        self.group_v1.is_some()
    }

    fn get_is_group_v2(&self) -> bool {
        self.group_v2.is_some()
    }

    fn member_count(&self) -> i32 {
        self.membership_list.pinned().borrow_mut().row_count()
    }

    fn get_valid(&self) -> bool {
        self.id.is_some() && (self.group_v1.is_some() || self.group_v2.is_some())
    }

    fn members(&self) -> QVariant {
        self.membership_list.pinned().into()
    }

    #[with_executor]
    fn set_group_id(&mut self, storage: Option<Storage>, id: QString) {
        self.id = Some(id.to_string());
        if let Some(storage) = storage {
            self.init(storage);
        }
    }

    fn init(&mut self, storage: Storage) {
        if let Some(id) = &self.id {
            self.group_v1 = None;
            self.group_v2 = None;
            if id.len() == 32 {
                self.group_v1 = storage.fetch_group_by_group_v1_id(id);
                self.membership_list
                    .pinned()
                    .borrow_mut()
                    .load_v1(storage, id);
            } else if id.len() == 64 {
                self.group_v2 = storage.fetch_group_by_group_v2_id(id);
                self.membership_list
                    .pinned()
                    .borrow_mut()
                    .load_v2(storage, id);
            } else {
                log::debug!("ID set to invalid length.  Leaving model empty.");
                self.membership_list.pinned().borrow_mut().clear();
            }
        }
    }
}

pub enum GroupMembership {
    V1(GroupV1Member),
    V2(GroupV2Member),
}

impl GroupMembership {
    fn member_since(&self) -> Option<NaiveDateTime> {
        match self {
            Self::V1(v1) => v1.member_since,
            Self::V2(v2) => Some(v2.member_since),
        }
    }

    fn role(&self) -> i32 {
        match self {
            Self::V1(_v1) => -1,
            Self::V2(v2) => v2.role,
        }
    }
}

#[derive(QObject, Default)]
pub struct GroupMembershipListModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<(GroupMembership, orm::Recipient)>,
}

impl GroupMembershipListModel {
    fn load_v1(&mut self, storage: Storage, id: &str) {
        self.begin_reset_model();
        self.content = storage
            .fetch_group_members_by_group_v1_id(id)
            .into_iter()
            .map(|(membership, member)| (GroupMembership::V1(membership), member))
            .collect();
        self.end_reset_model();
    }

    fn load_v2(&mut self, storage: Storage, id: &str) {
        self.begin_reset_model();
        self.content = storage
            .fetch_group_members_by_group_v2_id(id)
            .into_iter()
            .map(|(membership, member)| (GroupMembership::V2(membership), member))
            .collect();
        self.end_reset_model();
    }

    fn clear(&mut self) {
        self.begin_reset_model();
        self.content.clear();
        self.end_reset_model();
    }
}

define_model_roles! {
    enum GroupMembershipRoles for GroupMembership [with offset 100] {
        MemberSince(fn member_since(&self) via qdatetime_from_naive_option): "memberSince",
        Role(fn role(&self)): "role",
    }
}

impl QAbstractListModel for GroupMembershipListModel {
    fn row_count(&self) -> i32 {
        self.content.len() as _
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role > 100 {
            let role = GroupMembershipRoles::from(role);
            role.get(&self.content[index.row() as usize].0)
        } else {
            let role = RecipientRoles::from(role);
            role.get(&self.content[index.row() as usize].1)
        }
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        GroupMembershipRoles::role_names()
            .into_iter()
            .chain(RecipientRoles::role_names().into_iter())
            .collect()
    }
}
