use std::collections::BTreeMap;

use dleq_secret::{ElGamalPointCipher, PK, PreProof, Scalar};
use onlyhack_client::{GetProfilesResponse, OnlyhackClient, OnlyhackClientCtors, onlyhack::*};
use sails_rs::{Decode, Encode, client::*, gtest::*, hex};

pub const DEPLOYER_ID: u64 = 10;
pub const MODEL_ID: u64 = 42;
pub const FAN_ID: u64 = 43;
pub const STRANGER_ID: u64 = 44;

pub struct FanKeys {
    pub sk: Scalar,
    pub pk: dleq_secret::RistrettoPoint,
}

pub struct Sim {
    env: GtestEnv,
    program: Actor<onlyhack_client::OnlyhackClientProgram, GtestEnv>,
    service: Service<onlyhack_client::onlyhack::OnlyhackImpl, GtestEnv>,
    preproofs: BTreeMap<u64, PreProof>,
    prices: BTreeMap<u64, u128>,
}

impl Sim {
    pub async fn new() -> Result<Self, String> {
        let system = System::new();

        system.mint_to(DEPLOYER_ID, 100_000_000_000_000);
        system.mint_to(MODEL_ID, 100_000_000_000_000);
        system.mint_to(FAN_ID, 100_000_000_000_000);
        system.mint_to(STRANGER_ID, 100_000_000_000_000);

        let program_code_id = system.submit_code(onlyhack::WASM_BINARY);
        let env = GtestEnv::new(system, DEPLOYER_ID.into());
        let program = env
            .deploy::<onlyhack_client::OnlyhackClientProgram>(program_code_id, b"salt".to_vec())
            .create()
            .await
            .map_err(|e| format!("deploy/create failed: {e:?}"))?;

        let service = program.onlyhack();

        Ok(Self {
            env,
            program,
            service,
            preproofs: BTreeMap::new(),
            prices: BTreeMap::new(),
        })
    }

    pub async fn create_model_profile(&mut self, name: String, about: String) -> Result<(), String> {
        self.service
            .model_create_profile(name, about, vec!["free teaser".to_string()])
            .with_params(|p| p.with_actor_id(MODEL_ID.into()))
            .await
            .map_err(|e| format!("model_create_profile failed: {e:?}"))?;
        Ok(())
    }

    pub async fn add_paid_content(
        &mut self,
        preview: String,
        plaintext: Vec<u8>,
        price: u128,
    ) -> Result<u64, String> {
        let (public_data, pre_proof) = dleq_secret::pre_proof_and_public_for_message(&plaintext);
        let content_id = self
            .service
            .model_add_paid_content((preview, hex::encode(public_data.encode()), price))
            .with_params(|p| p.with_actor_id(MODEL_ID.into()))
            .await
            .map_err(|e| format!("model_add_paid_content failed: {e:?}"))?;

        self.preproofs.insert(content_id, pre_proof);
        self.prices.insert(content_id, price);
        Ok(content_id)
    }

    pub async fn get_profiles_for(&self, actor_id: u64) -> Result<GetProfilesResponse, String> {
        let profiles = self
            .service
            .get_profiles()
            .with_params(|p| p.with_actor_id(actor_id.into()))
            .await
            .map_err(|e| format!("get_profiles failed: {e:?}"))?;
        Ok(GetProfilesResponse {
            profiles: profiles.into_iter().map(Into::into).collect(),
        })
    }

    pub fn balances(&self) -> (u128, u128) {
        (
            self.env.system().balance_of(MODEL_ID),
            self.env.system().balance_of(FAN_ID),
        )
    }

    pub async fn buy_as_fan(&mut self, content_id: u64, fan_keys: &FanKeys) -> Result<(u128, String), String> {
        let price = *self
            .prices
            .get(&content_id)
            .ok_or_else(|| format!("unknown content id {content_id}"))?;

        self.service
            .buy_content(MODEL_ID.into(), content_id, hex::encode(PK(fan_keys.pk).encode()))
            .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(price))
            .send_one_way()
            .map_err(|e| format!("buy_content send failed: {e:?}"))?;

        let res = self.env.system().run_next_block();
        let mut pk_bytes = res
            .log
            .iter()
            .find(|log| log.destination() == MODEL_ID.into())
            .ok_or_else(|| "model did not receive grant request".to_string())?
            .payload();

        let pk = PK::decode(&mut pk_bytes).map_err(|e| format!("pk decode failed: {e:?}"))?;

        let pre_proof = self
            .preproofs
            .get(&content_id)
            .ok_or_else(|| format!("missing preproof for content id {content_id}"))?;
        let grant = pre_proof.proof_for_pk(pk.0);

        let log = Log::builder().dest(MODEL_ID).source(self.program.id());
        self.env
            .system()
            .get_mailbox(MODEL_ID)
            .reply(log, grant, 0)
            .map_err(|e| format!("reply grant failed: {e:?}"))?;

        let res2 = self.env.system().run_next_block();
        let mut payload = res2
            .log
            .iter()
            .find(|log| log.destination() == FAN_ID.into())
            .ok_or_else(|| "fan did not receive buy reply".to_string())?
            .payload();

        let tuple = <(String, String, Vec<u8>)>::decode(&mut payload)
            .map_err(|e| format!("buy reply tuple decode failed: {e:?}"))?;
        let cipher = Result::<ElGamalPointCipher, String>::decode(&mut tuple.2.as_slice())
            .map_err(|e| format!("buy result decode failed: {e:?}"))?
            .map_err(|e| format!("buy returned error: {e}"))?;

        let enc = self
            .service
            .get_enc_content(MODEL_ID.into(), content_id)
            .await
            .map_err(|e| format!("get_enc_content failed: {e:?}"))?;
        let dec = cipher.decrypt(enc, fan_keys.sk);

        let payout_log = Log::builder()
            .source(self.program.id())
            .dest(MODEL_ID)
            .payload_bytes([]);
        if self.env.system().get_mailbox(MODEL_ID).contains(&payout_log) {
            let _ = self.env.system().get_mailbox(MODEL_ID).claim_value(payout_log);
        }

        let decrypted = String::from_utf8(dec).unwrap_or_else(|_| "<non-utf8>".to_string());
        Ok((price, decrypted))
    }
}
