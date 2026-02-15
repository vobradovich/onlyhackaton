#![no_std]

use sails_rs::prelude::{collections::BTreeMap, *};

pub type ContentId = u64;
pub type EncryptedContent = String;
pub type PaidContentTuple = (String, u128);
pub type ProfileTuple = (
    String,
    String,
    Vec<(ContentId, String)>,
    Vec<(ContentId, PaidContentTuple)>,
);

#[derive(Clone, Debug, Default, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct PaidContent {
    pub preview: String,
    pub price: u128,
}

#[derive(Clone, Debug, Default, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct ModelProfile {
    pub name: String,
    pub about: String,
    pub open_content: BTreeMap<ContentId, String>,
    pub paid_content: BTreeMap<ContentId, PaidContent>,
}

#[derive(Default)]
struct State {
    models: BTreeMap<ActorId, ModelProfile>,
    buys: BTreeMap<(ActorId, ContentId), EncryptedContent>,
    content_next_id: ContentId,
}

static mut STATE: Option<State> = None;

#[allow(static_mut_refs)]
fn state() -> &'static State {
    unsafe { STATE.as_ref().expect("state is not initialized") }
}

#[allow(static_mut_refs)]
fn state_mut() -> &'static mut State {
    unsafe { STATE.as_mut().expect("state is not initialized") }
}

struct Onlyhack;

fn next_content_id(state: &mut State) -> ContentId {
    let content_id = state.content_next_id;
    state.content_next_id = state
        .content_next_id
        .checked_add(1)
        .expect("content id overflow");
    content_id
}

impl Onlyhack {
    pub fn new() -> Self {
        Self
    }
}

#[sails_rs::service]
impl Onlyhack {
    #[export]
    pub fn model_create_profile(
        &mut self,
        name: String,
        about: String,
        open_content: Vec<String>,
    ) {
        let model_id = Syscall::message_source();
        let state = state_mut();
        let mut open_content_map = BTreeMap::new();
        for content in open_content {
            let content_id = next_content_id(state);
            _ = open_content_map.insert(content_id, content);
        }

        let profile = ModelProfile {
            name,
            about,
            open_content: open_content_map,
            paid_content: BTreeMap::new(),
        };

        _ = state.models.insert(model_id, profile);
    }

    #[export]
    pub fn model_add_paid_content(&mut self, paid_content: Vec<PaidContentTuple>) {
        let model_id = Syscall::message_source();
        let state = state_mut();
        assert!(
            state.models.contains_key(&model_id),
            "profile is not created"
        );

        for (preview, price) in paid_content {
            let content_id = next_content_id(state);
            state
                .models
                .get_mut(&model_id)
                .expect("profile is not created")
                .paid_content
                .insert(
                    content_id,
                    PaidContent {
                        preview,
                        price,
                    },
                );
        }
    }

    #[export]
    pub fn get_profile(&self, model_id: ActorId) -> ProfileTuple {
        state()
            .models
            .get(&model_id)
            .map(|profile| {
                (
                    profile.name.clone(),
                    profile.about.clone(),
                    profile
                        .open_content
                        .iter()
                        .map(|(content_id, content)| (*content_id, content.clone()))
                        .collect(),
                    profile
                        .paid_content
                        .iter()
                        .map(|(content_id, paid_content)| {
                            (
                                *content_id,
                                (paid_content.preview.clone(), paid_content.price),
                            )
                        })
                        .collect(),
                )
            })
            .expect("profile is not created")
    }

    #[export(payable)]
    pub async fn buy_content(
        &mut self,
        model_id: ActorId,
        content_id: ContentId,
        _pkey: String,
    ) -> (bool, EncryptedContent, String) {
        let buyer_id = Syscall::message_source();
        let transferred = Syscall::message_value();

        let state = state_mut();
        if let Some(encoded_content) = state.buys.get(&(buyer_id, content_id)) {
            return (true, encoded_content.clone(), String::new());
        }

        let Some(profile) = state.models.get(&model_id) else {
            return (false, String::new(), "profile is not created".to_string());
        };
        let Some(paid_content) = profile.paid_content.get(&content_id) else {
            return (false, String::new(), "paid content does not exist".to_string());
        };
        if transferred != paid_content.price {
            return (false, String::new(), "invalid payment amount".to_string());
        }

        // TODO get encoded content by call `model_id`.
        let encrypted_content = String::new();
        state
            .buys
            .insert((buyer_id, content_id), encrypted_content.clone());

        (true, encrypted_content, String::new())
    }

    #[export]
    pub fn get_all_purchased_content(&self) -> Vec<(ContentId, EncryptedContent)> {
        let buyer_id = Syscall::message_source();
        state()
            .buys
            .iter()
            .filter(|((actor_id, _), _)| actor_id == &buyer_id)
            .map(|((_, content_id), encrypted_content)| (*content_id, encrypted_content.clone()))
            .collect()
    }
}

#[derive(Default)]
pub struct Program;

#[sails_rs::program(payable)]
impl Program {
    pub fn create() -> Self {
        unsafe {
            STATE = Some(State::default());
        }
        Self::default()
    }

    pub fn onlyhack(&self) -> Onlyhack {
        Onlyhack::new()
    }
}
