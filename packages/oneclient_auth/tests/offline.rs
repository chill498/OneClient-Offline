use chrono::{TimeDelta, Utc};
use oneclient_auth::{AccountKind, CredentialsStore, offline_account, offline_uuid};
use uuid::Uuid;

// The path override is process-wide. Serialise tests that save/load auth.json.
static AUTH_FILE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn isolate_launcher_dir() {
    oneclient_common::paths::set_launcher_dir(
        std::env::temp_dir().join(format!("oneclient-auth-test-{}", Uuid::new_v4())),
    );
}

fn fake_microsoft_account(username: &str) -> oneclient_auth::MinecraftAccount {
    let mut account = offline_account(username.to_string());
    account.kind = AccountKind::Microsoft;
    account.access_token = "access".into();
    account.refresh_token = "refresh".into();
    account
}

#[test]
fn offline_uuid_matches_vanilla_algorithm() {
    let id = offline_uuid("Notch");

    assert_eq!(
        id,
        Uuid::parse_str("b50ad385-829d-3141-a216-7e7d7539ba7f").unwrap()
    );
    assert_ne!(id, offline_uuid("Steve"));
}

#[test]
fn first_offline_account_needs_no_microsoft_and_becomes_default() {
    let mut store = CredentialsStore::default();
    let account = store.add_offline_account("Steve".into()).unwrap();

    assert_eq!(account.kind, AccountKind::Offline);
    assert_eq!(account.id, offline_uuid("Steve"));
    assert_eq!(store.default_user, Some(account.id));
    assert!(!store.has_microsoft_account());
    assert!(account.access_token.is_empty());
    assert!(account.refresh_token.is_empty());
}

#[test]
fn offline_account_allowed_when_microsoft_exists() {
    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    store.users.insert(msa.id, msa);

    let offline = store.add_offline_account("Steve".into()).unwrap();
    assert_eq!(offline.kind, AccountKind::Offline);
    assert_eq!(offline.username, "Steve");
}

#[tokio::test]
async fn default_offline_survives_when_last_microsoft_removed() {
    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    let offline = store.add_offline_account("Steve".into()).unwrap();
    store.default_user = Some(offline.id);
    store.users.remove(&msa_id);

    let account = store.default_account().await.unwrap().unwrap();
    assert_eq!(account.id, offline.id);
    assert!(account.is_offline());
    assert!(account.access_token.is_empty());
}

#[test]
fn adding_offline_account_keeps_an_existing_default() {
    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);
    store.default_user = Some(msa_id);

    store.add_offline_account("Steve".into()).unwrap();
    assert_eq!(store.default_user, Some(msa_id));
}

#[test]
fn invalid_and_duplicate_names_do_not_change_the_store() {
    let mut store = CredentialsStore::default();
    for name in [
        "",
        "ab",
        "abcdefghijklmnopq",
        "has space",
        "Steve!",
        "Stéve",
    ] {
        assert!(matches!(
            store.add_offline_account(name.into()),
            Err(oneclient_auth::AuthError::InvalidOfflineUsername { .. })
        ));
    }
    assert!(store.users.is_empty());
    assert!(store.default_user.is_none());

    let first = store.add_offline_account("Steve".into()).unwrap();
    assert!(matches!(
        store.add_offline_account("steve".into()),
        Err(oneclient_auth::AuthError::DuplicateUsername { .. })
    ));
    assert_eq!(store.users.len(), 1);
    assert_eq!(store.default_user, Some(first.id));
}

#[tokio::test]
async fn username_only_account_persists_without_tokens() {
    let _guard = AUTH_FILE_LOCK.lock().await;
    isolate_launcher_dir();
    let mut store = CredentialsStore::default();
    let account = store
        .add_offline_account_and_save("Steve".into())
        .await
        .unwrap();

    let mut reloaded = CredentialsStore::load().await.unwrap();
    let saved = reloaded.default_account().await.unwrap().unwrap();
    assert_eq!(saved.id, account.id);
    assert_eq!(saved.username, "Steve");
    assert!(saved.is_offline());
    assert!(saved.access_token.is_empty());
    assert!(saved.refresh_token.is_empty());
    assert!(!reloaded.has_microsoft_account());
}

#[test]
fn token_expiring_imminently_counts_as_expired() {
    let mut account = fake_microsoft_account("MsaUser");

    account.expires = Utc::now() + TimeDelta::hours(1);
    assert!(!account.is_expired(), "a token with an hour left is usable");

    account.expires = Utc::now() + TimeDelta::seconds(5);
    assert!(
        account.is_expired(),
        "a token about to lapse must be renewed before launching"
    );

    account.expires = Utc::now() - TimeDelta::hours(1);
    assert!(account.is_expired());
}

#[tokio::test]
async fn default_account_returns_expired_token_without_refreshing() {
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let mut msa = fake_microsoft_account("MsaUser");
    msa.expires = Utc::now() - TimeDelta::hours(5);
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);

    store.default_user = Some(msa_id);

    let account = store
        .default_account()
        .await
        .expect("an expired token must not fail the read path")
        .expect("the only account should be the default");

    assert_eq!(account.id, msa_id);
    assert!(account.is_expired());
}

#[tokio::test]
async fn signed_out_account_stays_listed_and_default() {
    let _guard = AUTH_FILE_LOCK.lock().await;
    isolate_launcher_dir();

    let mut store = CredentialsStore::default();
    let msa = fake_microsoft_account("MsaUser");
    let msa_id = msa.id;
    store.users.insert(msa_id, msa);
    store.default_user = Some(msa_id);

    store.mark_signed_out(msa_id).await.unwrap();

    let reloaded = CredentialsStore::load().await.unwrap();
    let account = reloaded.get_account(msa_id).expect("entry must survive");
    assert!(account.needs_sign_in());
    assert!(
        !account.refresh_token.is_empty(),
        "the refresh token is kept so the next renewal can retry it"
    );
    assert!(account.access_token.is_empty());
    assert_eq!(reloaded.default_user, Some(msa_id));
}

#[test]
fn accounts_saved_before_the_flag_existed_load_as_signed_in() {
    let account: oneclient_auth::MinecraftAccount = serde_json::from_str(&format!(
        r#"{{"id":"{}","username":"MsaUser","access_token":"a","refresh_token":"r","expires":"2026-01-01T00:00:00Z","kind":"microsoft"}}"#,
        Uuid::new_v4()
    ))
    .unwrap();

    assert!(!account.needs_sign_in());
}

#[test]
fn offline_accounts_never_need_sign_in() {
    let mut account = offline_account("Steve".into());
    account.signed_out = true;

    assert!(!account.needs_sign_in());
}
