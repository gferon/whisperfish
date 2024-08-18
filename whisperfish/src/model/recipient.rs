#![allow(non_snake_case)]

use crate::model::*;
use crate::store::observer::{EventObserving, Interest};
use crate::store::orm;
use actix::{ActorContext, Handler};
use futures::TryFutureExt;
use libsignal_service::protocol::SessionStore;
use libsignal_service::session_store::SessionStoreExt;
use libsignal_service::ServiceAddress;
use qmeta_async::with_executor;
use qmetaobject::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// QML-constructable object that interacts with a single recipient.
#[derive(Default, QObject)]
pub struct RecipientImpl {
    base: qt_base_class!(trait QObject),
    recipient_id: Option<i32>,
    // XXX What about PNI?
    recipient_uuid: Option<Uuid>,
    recipient: Option<RecipientWithAnalyzedSession>,
    fingerprint_needed: bool,

    force_init: bool,
}

crate::observing_model! {
    pub struct Recipient(RecipientImpl) {
        recipientId: i32; READ get_recipient_id WRITE set_recipient_id,
        recipientUuid: String; READ get_recipient_uuid WRITE set_recipient_uuid,
        fingerprintNeeded: bool; READ get_fingerprint_needed WRITE set_fingerprint_needed,
        valid: bool; READ get_valid,
    } WITH OPTIONAL PROPERTIES FROM recipient WITH ROLE RecipientWithAnalyzedSessionRoles {
        id Id,
        externalId ExternalId,
        directMessageSessionId DirectMessageSessionId,
        uuid Uuid,
        // These two are aliases
        e164 E164,
        phoneNumber PhoneNumber,
        username Username,
        email Email,

        sessionFingerprint SessionFingerprint,
        sessionIsPostQuantum SessionIsPostQuantum,

        blocked Blocked,

        name JoinedName,
        familyName FamilyName,
        givenName GivenName,

        about About,
        emoji Emoji,

        unidentifiedAccessMode UnidentifiedAccessMode,
        profileSharing ProfileSharing,

        isRegistered IsRegistered,
    }
}

impl EventObserving for RecipientImpl {
    type Context = ModelContext<Self>;

    fn observe(&mut self, ctx: Self::Context, _event: crate::store::observer::Event) {
        tracing::trace!("Observer recipient re-init");
        self.force_init = true;
        self.init(ctx);
    }

    fn interests(&self) -> Vec<Interest> {
        self.recipient
            .iter()
            .flat_map(|r| r.inner.interests())
            .collect()
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct SessionAnalyzed {
    recipient_id: i32,
    fingerprint: String,
    versions: Vec<(u32, u32)>,
}

impl Handler<SessionAnalyzed> for ObservingModelActor<RecipientImpl> {
    type Result = ();

    fn handle(
        &mut self,
        SessionAnalyzed {
            recipient_id,
            fingerprint,
            versions,
        }: SessionAnalyzed,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match self.model.upgrade() {
            Some(model) => {
                let model = model.pinned();
                let mut model = model.borrow_mut();
                if let Some(recipient) = &mut model.recipient {
                    if recipient.id == recipient_id {
                        recipient.fingerprint = Some(fingerprint);
                        recipient.versions = versions;
                        // TODO: trigger something changed
                    }
                }
            }
            None => {
                // In principle, the actor should have gotten stopped when the model got dropped,
                // because the actor's only strong reference is contained in the ObservingModel.
                tracing::debug!("Model got dropped, stopping actor execution.");
                // XXX What is the difference between stop and terminate?
                ctx.stop();
            }
        }
    }
}

impl RecipientImpl {
    fn get_recipient_id(&self) -> i32 {
        self.recipient_id.unwrap_or(-1)
    }

    fn get_recipient_uuid(&self) -> String {
        self.recipient_uuid
            .as_ref()
            .map(Uuid::to_string)
            .unwrap_or("".into())
    }

    fn get_valid(&self) -> bool {
        self.recipient_id.is_some() && self.recipient.is_some()
    }

    #[with_executor]
    #[tracing::instrument(skip(self, ctx))]
    fn set_recipient_id(&mut self, ctx: Option<ModelContext<Self>>, id: i32) {
        if self.recipient_id == Some(id) {
            return;
        }
        self.recipient_id = Some(id);
        self.recipient_uuid = None; // Set in init()
        if let Some(ctx) = ctx {
            self.init(ctx);
        }
    }

    #[with_executor]
    #[tracing::instrument(skip(self, ctx))]
    fn set_recipient_uuid(&mut self, ctx: Option<ModelContext<Self>>, uuid: String) {
        if self.recipient_uuid.map(|u| u.to_string()).as_ref() == Some(&uuid) {
            return;
        }
        self.recipient_id = None; // Set in init()
        if let Ok(uuid) = Uuid::parse_str(&uuid) {
            self.recipient_uuid = Some(uuid);
        } else {
            tracing::warn!("QML requested unparsable UUID");
            self.recipient_uuid = None;
        }
        if let Some(ctx) = ctx {
            self.init(ctx);
        }
    }

    #[with_executor]
    #[tracing::instrument(skip(self, ctx))]
    fn set_fingerprint_needed(&mut self, ctx: Option<ModelContext<Self>>, needed: bool) {
        self.fingerprint_needed = needed;
        if let Some(ctx) = ctx {
            self.compute_fingerprint(ctx);
        }
    }

    fn get_fingerprint_needed(&self) -> bool {
        self.fingerprint_needed
    }

    fn init(&mut self, ctx: ModelContext<Self>) {
        if self.recipient.is_none() || self.force_init {
            let storage = ctx.storage();
            let recipient = if let Some(uuid) = self.recipient_uuid {
                storage
                    .fetch_recipient(&ServiceAddress::new_aci(uuid))
                    .map(|inner| {
                        let direct_message_recipient_id = storage
                            .fetch_session_by_recipient_id(inner.id)
                            .map(|session| session.id)
                            .unwrap_or(-1);
                        self.recipient_id = Some(inner.id);
                        // XXX trigger Qt signal for this?
                        RecipientWithAnalyzedSession {
                            inner,
                            direct_message_recipient_id,
                            fingerprint: None,
                            versions: Vec::new(),
                        }
                    })
            } else if let Some(id) = self.recipient_id {
                if id >= 0 {
                    storage.fetch_recipient_by_id(id).map(|inner| {
                        let direct_message_recipient_id = storage
                            .fetch_session_by_recipient_id(inner.id)
                            .map(|session| session.id)
                            .unwrap_or(-1);
                        // XXX Clean this up after #532
                        self.recipient_uuid = inner.uuid.or(Some(Uuid::nil()));
                        // XXX trigger Qt signal for this?
                        RecipientWithAnalyzedSession {
                            inner,
                            direct_message_recipient_id,
                            fingerprint: None,
                            versions: Vec::new(),
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            };

            // XXX trigger Qt signal for this?
            self.recipient = recipient;
        }

        if self.force_init {
            self.force_init = false;
            if let Some(recipient) = self.recipient.as_mut() {
                recipient.fingerprint = None;
            }
        }

        if self.fingerprint_needed {
            self.compute_fingerprint(ctx);
        }
    }

    fn compute_fingerprint(&mut self, ctx: ModelContext<Self>) {
        if self.recipient.is_none() || self.recipient.as_ref().unwrap().fingerprint.is_some() {
            tracing::trace!("Not computing fingerprint");
            return;
        }

        tracing::trace!("Computing fingerprint");
        // If an ACI recipient was found, attempt to compute the fingerprint
        let recipient = self.recipient.as_ref().unwrap();
        if let Some(recipient_svc) = recipient.to_aci_service_address() {
            let storage = ctx.storage();
            let recipient_id = recipient.id;
            let compute = async move {
                let local_svc = storage.fetch_self_service_address_aci().expect("self ACI");
                let fingerprint = storage
                    .aci_storage()
                    .compute_safety_number(&local_svc, &recipient_svc)
                    .await?;
                let sessions = storage
                    .aci_storage()
                    .get_sub_device_sessions(&recipient_svc)
                    .await?;
                let mut versions = Vec::new();
                for device_id in sessions {
                    let session = storage
                        .aci_storage()
                        .load_session(&recipient_svc.to_protocol_address(device_id))
                        .await?;
                    let version = session
                        .map(|x| x.session_version())
                        .transpose()?
                        .unwrap_or(0);
                    versions.push((device_id, version));
                }
                ctx.addr()
                    .send(SessionAnalyzed {
                        recipient_id,
                        fingerprint,
                        versions,
                    })
                    .await?;

                Result::<_, anyhow::Error>::Ok(())
            }
            .map_ok_or_else(|e| tracing::error!("Computing fingerprint: {}", e), |_| ());
            actix::spawn(compute);
        }
    }
}

#[derive(QObject, Default)]
pub struct RecipientListModel {
    base: qt_base_class!(trait QAbstractListModel),
    content: Vec<orm::Recipient>,
}

pub struct RecipientWithAnalyzedSession {
    inner: orm::Recipient,
    direct_message_recipient_id: i32,
    fingerprint: Option<String>,
    versions: Vec<(u32, u32)>,
}

impl RecipientWithAnalyzedSession {
    fn session_is_post_quantum(&self) -> bool {
        const KYBER_AWARE_MESSAGE_VERSION: u32 = 4;

        self.versions
            .iter()
            .all(|(_, version)| *version >= KYBER_AWARE_MESSAGE_VERSION)
    }
}

impl std::ops::Deref for RecipientWithAnalyzedSession {
    type Target = orm::Recipient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RecipientListModel {}

define_model_roles! {
    pub(super) enum RecipientWithAnalyzedSessionRoles for RecipientWithAnalyzedSession {
        Id(id): "id",
        ExternalId(external_id via qstring_from_option): "externalId",
        DirectMessageSessionId(direct_message_recipient_id): "directMessageSessionId",
        Uuid(uuid via qstring_from_optional_to_string): "uuid",
        // These two are aliases
        E164(e164 via qstring_from_optional_to_string): "e164",
        PhoneNumber(e164 via qstring_from_optional_to_string): "phoneNumber",
        Username(username via qstring_from_option): "username",
        Email(email via qstring_from_option): "email",
        IsRegistered(is_registered): "isRegistered",

        Blocked(blocked): "blocked",

        JoinedName(profile_joined_name via qstring_from_option): "name",
        FamilyName(profile_family_name via qstring_from_option): "familyName",
        GivenName(profile_given_name via qstring_from_option): "givenName",

        About(about via qstring_from_option): "about",
        Emoji(about_emoji via qstring_from_option): "emoji",

        UnidentifiedAccessMode(unidentified_access_mode via Into<i32>::into): "unidentifiedAccessMode",
        ProfileSharing(profile_sharing): "profileSharing",

        SessionFingerprint(fingerprint via qstring_from_option): "sessionFingerprint",

        SessionIsPostQuantum(fn session_is_post_quantum(&self)): "sessionIsPostQuantum",
    }
}

define_model_roles! {
    pub(super) enum RecipientRoles for orm::Recipient {
        Id(id): "id",
        ExternalId(external_id via qstring_from_option): "externalId",
        Uuid(uuid via qstring_from_optional_to_string): "uuid",
        // These two are aliases
        E164(e164 via qstring_from_optional_to_string): "e164",
        PhoneNumber(e164 via qstring_from_optional_to_string): "phoneNumber",
        Username(username via qstring_from_option): "username",
        Email(email via qstring_from_option): "email",

        Blocked(blocked): "blocked",

        JoinedName(profile_joined_name via qstring_from_option): "name",
        FamilyName(profile_family_name via qstring_from_option): "familyName",
        GivenName(profile_given_name via qstring_from_option): "givenName",

        About(about via qstring_from_option): "about",
        Emoji(about_emoji via qstring_from_option): "emoji",

        UnidentifiedAccessMode(unidentified_access_mode via Into<i32>::into): "unidentifiedAccessMode",
        ProfileSharing(profile_sharing): "profileSharing",

        IsRegistered(is_registered): "isRegistered",
    }
}

impl QAbstractListModel for RecipientListModel {
    fn row_count(&self) -> i32 {
        self.content.len() as _
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let role = RecipientRoles::from(role);
        role.get(&self.content[index.row() as usize])
    }

    fn role_names(&self) -> HashMap<i32, QByteArray> {
        RecipientRoles::role_names()
    }
}
