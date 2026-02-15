use onlyhack_client::{OnlyhackClient, OnlyhackClientCtors, onlyhack::*};
use sails_rs::{client::*, gtest::*};

const DEPLOYER_ID: u64 = 10;
const MODEL_ID: u64 = 42;
const FAN_ID: u64 = 43;
const STRANGER_ID: u64 = 44;

#[tokio::test]
async fn mvp_purchase_flow_works() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
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

    service_client
        .model_add_paid_content(vec![("preview".to_string(), 55)])
        .with_params(|p| p.with_actor_id(MODEL_ID.into()))
        .await
        .unwrap();

    let failed_purchase = service_client
        .buy_content(MODEL_ID.into(), 1, "user-pkey".to_string())
        .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(54))
        .await
        .unwrap();
    assert_eq!(
        failed_purchase,
        (false, String::new(), "invalid payment amount".to_string())
    );

    let overpaid_purchase = service_client
        .buy_content(MODEL_ID.into(), 1, "user-pkey".to_string())
        .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(56))
        .await
        .unwrap();
    assert_eq!(
        overpaid_purchase,
        (false, String::new(), "invalid payment amount".to_string())
    );
    let before_buy = service_client
        .get_all_purchased_content()
        .with_params(|p| p.with_actor_id(FAN_ID.into()))
        .await
        .unwrap();
    assert!(before_buy.is_empty());

    let purchased = service_client
        .buy_content(MODEL_ID.into(), 1, "user-pkey".to_string())
        .with_params(|p| p.with_actor_id(FAN_ID.into()).with_value(55))
        .await
        .unwrap();

    // Current contract has TODO placeholder and returns empty bytes.
    assert_eq!(purchased, (true, String::new(), String::new()));

    // Same buyer/content should return cached buy result with no additional payment.
    let second = service_client
        .buy_content(MODEL_ID.into(), 1, "user-pkey-2".to_string())
        .with_params(|p| p.with_actor_id(FAN_ID.into()))
        .await
        .unwrap();
    assert_eq!(second, (true, String::new(), String::new()));

    let all_purchases = service_client
        .get_all_purchased_content()
        .with_params(|p| p.with_actor_id(FAN_ID.into()))
        .await
        .unwrap();
    assert_eq!(all_purchases.len(), 1);
    assert_eq!(all_purchases[0].0, 1);
    assert_eq!(all_purchases[0].1, String::new());

    let profile = service_client.get_profile(MODEL_ID.into()).await.unwrap();
    assert_eq!(profile.0, "alice".to_string());
    assert!(profile.2.iter().any(|(content_id, _)| *content_id == 0));
    assert!(profile.3.iter().any(|(content_id, _)| *content_id == 1));

    // Non-owner cannot modify model content.
    let unauthorized_add = service_client
        .model_add_paid_content(vec![("stranger-preview".to_string(), 1)])
        .with_params(|p| p.with_actor_id(STRANGER_ID.into()))
        .await;
    assert!(unauthorized_add.is_err());
}
