use dleq_secret::{ElGamalPointCipher, PK};
use onlyhack_client::{OnlyhackClient, OnlyhackClientCtors, onlyhack::*};
use sails_rs::{Decode, Encode, client::*, gtest::*, hex};

const DEPLOYER_ID: u64 = 10;
const MODEL_ID: u64 = 42;
const FAN_ID: u64 = 43;
const STRANGER_ID: u64 = 44;

#[tokio::test]
async fn mvp_purchase_flow_works() {
    let system = System::new();

    system.init_logger_with_default_filter("gwasm=debug,gtest=debug,sails_rs=debug");
    system.mint_to(DEPLOYER_ID, 100_000_000_000_000);
    system.mint_to(MODEL_ID, 100_000_000_000_000);
    system.mint_to(FAN_ID, 100_000_000_000_000);
    system.mint_to(STRANGER_ID, 100_000_000_000_000);
    // Submit program code into the system
    let program_code_id = system.submit_code(onlyhack::WASM_BINARY);

    // Create Sails Env
    let env = GtestEnv::new(system, DEPLOYER_ID.into());

    let program = env
        .deploy::<onlyhack_client::OnlyhackClientProgram>(program_code_id, b"salt".to_vec())
        .create() // Call program's constructor
        .await
        .unwrap();

    let mut service_client = program.onlyhack();

    service_client
        .model_create_profile(
            "alice".into(),
            "model".into(),
            vec!["free-teaser".to_string()],
        )
        .with_params(|p| p.with_actor_id(MODEL_ID.into()))
        .await
        .unwrap();

    let content = b"this is the secret content".to_vec();
    let (public_data, pre_proof) = dleq_secret::pre_proof_and_public_for_message(&content);

    let content_id = service_client
        .model_add_paid_content((
            "my secret content".to_string(),
            hex::encode(public_data.encode()),
            55,
        ))
        .with_params(|p| p.with_actor_id(MODEL_ID.into()))
        .await
        .unwrap();

    let user = dleq_secret::gen_keypair();

    let _message_id = service_client
        .buy_content(
            MODEL_ID.into(),
            content_id,
            hex::encode(PK(user.pk).encode()),
        )
        .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(55))
        .send_one_way()
        .unwrap();

    let res = env.system().run_next_block();

    let pk = PK::decode(
        &mut res
            .log
            .iter()
            .find(|log| log.destination() == MODEL_ID.into())
            .expect("model should receive a message")
            .payload(),
    )
    .unwrap();

    let log = Log::builder().dest(MODEL_ID).source(program.id());

    let grant = pre_proof.proof_for_pk(pk.0);
    env.system()
        .get_mailbox(MODEL_ID)
        .reply(log, grant, 0)
        .unwrap();

    let res = env.system().run_next_block();
    let mut bytes = res
        .log
        .iter()
        .find(|log| log.destination() == FAN_ID.into())
        .expect("fan should receive a message")
        .payload();

    let bytes = <(String, String, Vec<u8>)>::decode(&mut bytes).unwrap();
    let x = Result::<ElGamalPointCipher, String>::decode(&mut bytes.2.as_slice()).unwrap();

    println!("Point: {x:?}");


    let enc = service_client.get_enc_content(MODEL_ID.into(), content_id).await.unwrap();

    let dec_content = x.unwrap().decrypt(enc, user.sk);
    assert_eq!(dec_content, content);

    println!("Decrypted content: {:?}", String::from_utf8(dec_content).unwrap());

    // let failed_purchase = service_client
    //     .buy_content(MODEL_ID.into(), 1, "user-pkey".to_string())
    //     .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(54))
    //     .await
    //     .unwrap();
    // assert_eq!(
    //     failed_purchase,
    //     (false, String::new(), "invalid payment amount".to_string())
    // );

    // let overpaid_purchase = service_client
    //     .buy_content(MODEL_ID.into(), 1, "user-pkey".to_string())
    //     .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(56))
    //     .await
    //     .unwrap();
    // assert_eq!(
    //     overpaid_purchase,
    //     (false, String::new(), "invalid payment amount".to_string())
    // );
    // let before_buy = service_client
    //     .get_all_purchased_content()
    //     .with_params(|p| p.with_actor_id(FAN_ID.into()))
    //     .await
    //     .unwrap();
    // assert!(before_buy.is_empty());

    // let purchased = service_client
    //     .buy_content(MODEL_ID.into(), 1, "user-pkey".to_string())
    //     .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(55))
    //     .await
    //     .unwrap();

    // // Current contract has TODO placeholder and returns empty bytes.
    // assert_eq!(purchased, (true, String::new(), String::new()));

    // // Same buyer/content should return cached buy result with no additional payment.
    // let second = service_client
    //     .buy_content(MODEL_ID.into(), 1, "user-pkey-2".to_string())
    //     .with_params(|p| p.with_actor_id(FAN_ID.into()))
    //     .await
    //     .unwrap();
    // assert_eq!(second, (true, String::new(), String::new()));

    // let all_purchases = service_client
    //     .get_all_purchased_content()
    //     .with_params(|p| p.with_actor_id(FAN_ID.into()))
    //     .await
    //     .unwrap();
    // assert_eq!(all_purchases.len(), 1);
    // assert_eq!(all_purchases[0].0, 1);
    // assert_eq!(all_purchases[0].1, String::new());

    // let profile = service_client.get_profile(MODEL_ID.into()).await.unwrap();
    // assert_eq!(profile.0, "alice".to_string());
    // assert!(profile.2.iter().any(|(content_id, _)| *content_id == 0));
    // assert!(profile.3.iter().any(|(content_id, _)| *content_id == 1));

    // // Non-owner cannot modify model content.
    // let unauthorized_add = service_client
    //     .model_add_paid_content(vec![("stranger-preview".to_string(), 1)])
    //     .with_params(|p| p.with_actor_id(STRANGER_ID.into()))
    //     .await;
    // assert!(unauthorized_add.is_err());
}
