mod common;
use common::prelude::*;

// --- Chat Query Tests ---

#[tokio::test]
async fn test_list_chats_with_seed_data() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                meta { ResponseCode }
                affectedRows {
                    id name image createdat updatedat
                    chatparticipants { userid username slug img hasaccess }
                    chatmessages { id senderid chatid content createdat }
                }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listChats"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11801");
    let rows = data["affectedRows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    // Should be sorted by updatedat desc (group chat more recent)
    assert_eq!(rows[0]["id"].as_str().unwrap(), SEED_CHAT_GROUP.to_string());
    assert_eq!(
        rows[1]["id"].as_str().unwrap(),
        SEED_CHAT_PRIVATE.to_string()
    );
}

#[tokio::test]
async fn test_list_chats_no_chats() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"query { listChats { meta { ResponseCode } affectedRows { id } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listChats"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "21801");
    assert!(data["affectedRows"].is_null());
}

#[tokio::test]
async fn test_list_chats_without_auth() {
    let res = graphql(r#"query { listChats { meta { ResponseCode } } }"#).await;

    let data = &res["data"]["listChats"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_chat_includes_correct_participants() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id chatparticipants { userid } }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let group = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_GROUP.to_string())
        .unwrap();
    let participants: Vec<&str> = group["chatparticipants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["userid"].as_str().unwrap())
        .collect();
    assert_eq!(participants.len(), 3);
    assert!(participants.contains(&SEED_USER_VERIFIED.to_string().as_str()));
    assert!(participants.contains(&SEED_USER_ALICE.to_string().as_str()));
    assert!(participants.contains(&SEED_USER_BOB.to_string().as_str()));
}

#[tokio::test]
async fn test_chat_messages_ordered_by_time() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id chatmessages { createdat } }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let private = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let msgs = private["chatmessages"].as_array().unwrap();
    assert!(msgs.len() >= 2);
    for pair in msgs.windows(2) {
        assert!(pair[0]["createdat"].as_str().unwrap() <= pair[1]["createdat"].as_str().unwrap());
    }
}

#[tokio::test]
async fn test_private_chat_no_name_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id name image }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let private = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    assert!(private["name"].is_null());
    assert!(private["image"].is_null());
}

#[tokio::test]
async fn test_group_chat_has_name() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id name }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let group = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_GROUP.to_string())
        .unwrap();
    assert_eq!(group["name"].as_str().unwrap(), "Rust Developers");
}

// --- Chat Mutation Tests ---

#[tokio::test]
async fn test_create_private_chat() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createChat(input: {{ name: "dm", recipients: ["{SEED_USER_DAVE}"] }}) {{ meta {{ ResponseCode }} affectedRows {{ chatid }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11802");
    assert!(!data["affectedRows"]["chatid"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_private_chat_already_exists() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Try to create a chat between verified_user and alice — already exists as SEED_CHAT_PRIVATE
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createChat(input: {{ name: "dm", recipients: ["{SEED_USER_ALICE}"] }}) {{ meta {{ ResponseCode }} affectedRows {{ chatid }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11803");
    assert_eq!(
        data["affectedRows"]["chatid"].as_str().unwrap(),
        SEED_CHAT_PRIVATE.to_string()
    );
}

#[tokio::test]
async fn test_create_group_chat() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createChat(input: {{ name: "New Group", recipients: ["{SEED_USER_ALICE}", "{SEED_USER_CAROL}"] }}) {{ meta {{ ResponseCode }} affectedRows {{ chatid }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11802");
}

#[tokio::test]
async fn test_create_chat_no_recipients() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { createChat(input: { name: "empty", recipients: [] }) { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30301");
}

#[tokio::test]
async fn test_create_chat_without_auth() {
    let res = graphql(
        &format!(
            r#"mutation {{ createChat(input: {{ name: "dm", recipients: ["{SEED_USER_ALICE}"] }}) {{ meta {{ ResponseCode }} }} }}"#
        ),
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_send_chat_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "Hello!") {{ meta {{ ResponseCode }} affectedRows {{ id senderid chatid content createdat }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11804");
    let msg = &data["affectedRows"];
    assert_eq!(msg["content"].as_str().unwrap(), "Hello!");
    assert_eq!(
        msg["senderid"].as_str().unwrap(),
        SEED_USER_VERIFIED.to_string()
    );
    assert_eq!(
        msg["chatid"].as_str().unwrap(),
        SEED_CHAT_PRIVATE.to_string()
    );
}

#[tokio::test]
async fn test_send_message_nonexistent_chat() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { sendChatMessage(chatid: "ffffffff-ffff-4fff-afff-ffffffffffff", content: "test") { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30304");
}

#[tokio::test]
async fn test_send_message_not_participant() {
    let state = default_shared_state();
    // Carol is not in SEED_CHAT_PRIVATE
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "sneaky") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30305");
}

#[tokio::test]
async fn test_send_empty_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30307");
}

#[tokio::test]
async fn test_send_message_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let long_msg = "x".repeat(501);
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "{long_msg}") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30306");
}

#[tokio::test]
async fn test_send_message_without_auth() {
    let res = graphql(
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "test") {{ meta {{ ResponseCode }} }} }}"#
        ),
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_send_message_updates_chat_updatedat() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Get original updatedat
    let res1 = graphql_with_auth(
        &state,
        r#"query { listChats { affectedRows { id updatedat } } }"#,
        &token,
    )
    .await;

    let rows1 = res1["data"]["listChats"]["affectedRows"]
        .as_array()
        .unwrap();
    let private1 = rows1
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let old_updated = private1["updatedat"].as_str().unwrap().to_string();

    // Send a message
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "new msg") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Check updated timestamp
    let res2 = graphql_with_auth(
        &state,
        r#"query { listChats { affectedRows { id updatedat } } }"#,
        &token,
    )
    .await;

    let rows2 = res2["data"]["listChats"]["affectedRows"]
        .as_array()
        .unwrap();
    let private2 = rows2
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let new_updated = private2["updatedat"].as_str().unwrap();
    assert!(new_updated > old_updated.as_str());
}

#[tokio::test]
async fn test_sent_message_appears_in_list() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "new visible msg") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id chatmessages { content } }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let private = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let msgs = private["chatmessages"].as_array().unwrap();
    let has_new = msgs
        .iter()
        .any(|m| m["content"].as_str().unwrap() == "new visible msg");
    assert!(has_new);
}
