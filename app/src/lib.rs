#![no_std]

use dleq_secret::{ElGamalPointCipher, GrantWithProof, PK, PublicData};
use sails_rs::{
    gstd,
    prelude::{collections::BTreeMap, *},
};
use scale_info::build::state;

pub type ContentId = u64;
pub type PaidContentTuple = (String, String, u128);
pub type ProfileTuple = (
    String,
    String,
    Vec<(ContentId, String)>,
    Vec<(ContentId, PaidContentTuple)>,
);

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
struct OpenContentInfo {
    content_id: ContentId,
    preview: String,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
struct HiddenContentInfo {
    content_id: ContentId,
    preview: String,
    price: u128,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
struct PurchasedContentInfo {
    content_id: ContentId,
    preview: String,
    price: u128,
    enc_content: Vec<u8>,
    m_under_pk: ElGamalPointCipher,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
struct ProfileInfo {
    name: String,
    about: String,
    open_content: Vec<OpenContentInfo>,
    purchased_content: Vec<PurchasedContentInfo>,
    hidden_content: Vec<HiddenContentInfo>,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct PaidContent {
    pub preview: String,
    pub public_data: PublicData,
    pub price: u128,
}

#[derive(Clone, Debug, Default, Encode, Decode, TypeInfo)]
pub struct ModelProfile {
    pub name: String,
    pub about: String,
    pub open_content: BTreeMap<ContentId, String>,
    pub paid_content: BTreeMap<ContentId, PaidContent>,
}

#[derive(Default)]
struct State {
    models: BTreeMap<ActorId, ModelProfile>,
    buys: BTreeMap<(ActorId, ContentId), ElGamalPointCipher>,
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
    pub fn model_create_profile(&mut self, name: String, about: String, open_content: Vec<String>) {
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
    pub fn model_add_paid_content(&mut self, paid_content: PaidContentTuple) -> ContentId {
        let model_id = Syscall::message_source();
        let state = state_mut();
        assert!(
            state.models.contains_key(&model_id),
            "profile is not created"
        );

        let (preview, data_bytes, price) = paid_content;

        assert!(price > 0, "price must be greater than 0");

        let content_id = next_content_id(state);
        let data_bytes = hex::decode(data_bytes).expect("Failed to decode hex data");
        let public_data =
            PublicData::decode(&mut data_bytes.as_slice()).expect("Failed to decode data");
        state
            .models
            .get_mut(&model_id)
            .expect("profile is not created")
            .paid_content
            .insert(
                content_id,
                PaidContent {
                    preview,
                    public_data,
                    price,
                },
            );

        content_id
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
                                (
                                    paid_content.preview.clone(),
                                    hex::encode(paid_content.public_data.encode()),
                                    paid_content.price,
                                ),
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
        pk_bytes: String,
    ) -> CommandReply<Vec<u8>> {
        let ok = |p: ElGamalPointCipher| CommandReply::new((Ok(p) as Result<_, String>).encode());
        let err = |e: String| CommandReply::new((Err(e) as Result<ElGamalPointCipher, _>).encode());

        let transferred = Syscall::message_value();
        assert!(transferred > 0, "Payment value must be greater than 0");

        let pk = PK::decode(
            &mut hex::decode(pk_bytes)
                .expect("Failed to decode hex public key")
                .as_slice(),
        )
        .expect("Failed to decode public key");

        let buyer_id = Syscall::message_source();

        let state = state_mut();
        if let Some(point_cipher) = state.buys.get(&(buyer_id, content_id)) {
            return ok(point_cipher.clone()).with_value(transferred);
        }

        let profile = state
            .models
            .get(&model_id)
            .expect("profile for model not found");

        let paid_content = profile
            .paid_content
            .get(&content_id)
            .expect("paid content not found");

        if transferred < paid_content.price {
            panic!(
                "not enough payment, got {}, expected {}",
                transferred, paid_content.price
            );
        }

        let grant_bytes = match gstd::msg::send_for_reply(model_id, &pk, 0)
            .expect("failed to send request for grant from model")
            .await
        {
            Ok(bytes) => bytes,
            Err(e) => {
                return err(format!("failed to receive grant from model: {:?}", e))
                    .with_value(transferred);
            }
        };

        let grant = match GrantWithProof::decode(&mut grant_bytes.as_slice()) {
            Ok(grant) => grant,
            Err(e) => {
                return err(format!("failed to decode grant with proof: {:?}", e))
                    .with_value(transferred);
            }
        };

        // Verify the grant proof.
        if !grant.verify(&paid_content.public_data, pk.0) {
            return err("invalid grant proof".to_string()).with_value(transferred);
        }

        state
            .buys
            .insert((buyer_id, content_id), grant.enc_m_under_pk.clone());

        if gstd::msg::send(model_id, b"", transferred).is_err() {
            // Failed to send the payment ... so left the value in the contract for now
        }

        ok(grant.enc_m_under_pk)
    }

    #[export]
    pub fn get_enc_content(&self, model_id: ActorId, content_id: ContentId) -> Vec<u8> {
        let state = state_mut();

        let profile = state
            .models
            .get(&model_id)
            .expect("profile for model not found");

        let paid_content = profile
            .paid_content
            .get(&content_id)
            .expect("paid content not found");

        paid_content.public_data.enc_x.clone()
    }

    // #[export]
    // pub fn get_profiles(&self) -> Vec<ProfileInfo> {
    //     let state = state();

    //     let mut output = vec![];
    //     for model in state.models.iter() {
    //     }
    // }

    // #[export]
    // pub fn get_all_purchased_content(&self) -> Vec<(ContentId, EncryptedContent)> {
    //     let buyer_id = Syscall::message_source();
    //     state()
    //         .buys
    //         .iter()
    //         .filter(|((actor_id, _), _)| actor_id == &buyer_id)
    //         .map(|((_, content_id), encrypted_content)| (*content_id, encrypted_content.clone()))
    //         .collect()
    // }
}

#[derive(Default)]
pub struct Program;

#[sails_rs::program(payable)]
impl Program {
    pub fn create() -> Self {
        unsafe {
            STATE = Some(State::default());
        }

        Self
    }

    pub fn onlyhack(&self) -> Onlyhack {
        Onlyhack::new()
    }
}
