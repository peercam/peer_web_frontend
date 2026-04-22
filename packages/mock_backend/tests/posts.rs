mod common;
use common::prelude::*;

#[tokio::test]
async fn test_list_posts_with_seed_data() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 10) {
                meta { status ResponseCode }
                counter
                affectedRows {
                    id contenttype title amountlikes amountviews
                    isliked isviewed tags
                    user { id username slug }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    // 8 seed posts minus 1 ad post = 7 in normal feed
    assert!(data["counter"].as_i64().unwrap() >= 7);
    assert!(!data["affectedRows"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_posts_pagination() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 3) {
                meta { ResponseCode }
                counter
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 3);
    assert!(data["counter"].as_i64().unwrap() > 3);
}

#[tokio::test]
async fn test_list_posts_offset_beyond_range() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 100, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "21518");
    assert!(data["affectedRows"].as_array().unwrap().is_empty());
    assert!(data["counter"].as_i64().unwrap() >= 7);
}

#[tokio::test]
async fn test_list_posts_filter_by_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(filterBy: [IMAGE], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { id contenttype }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["contenttype"], "image");
    }
}

#[tokio::test]
async fn test_list_posts_filter_by_text() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(filterBy: [TEXT], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { id contenttype }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["contenttype"], "text");
    }
}

#[tokio::test]
async fn test_list_posts_sort_by_newest() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 20) {
                affectedRows { createdat }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    for i in 1..rows.len() {
        assert!(
            rows[i - 1]["createdat"].as_str().unwrap() >= rows[i]["createdat"].as_str().unwrap()
        );
    }
}

#[tokio::test]
async fn test_list_posts_sort_by_likes() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: LIKES, offset: 0, limit: 20) {
                affectedRows { amountlikes }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    for i in 1..rows.len() {
        assert!(
            rows[i - 1]["amountlikes"].as_i64().unwrap()
                >= rows[i]["amountlikes"].as_i64().unwrap()
        );
    }
}

#[tokio::test]
async fn test_list_posts_search_by_title() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(title: "rust", sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { title }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(
            row["title"]
                .as_str()
                .unwrap()
                .to_lowercase()
                .contains("rust")
        );
    }
}

#[tokio::test]
async fn test_list_posts_filter_by_tag() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(tag: "tutorial", sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { tags }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        let tags: Vec<&str> = row["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap())
            .collect();
        assert!(tags.contains(&"tutorial"));
    }
}

#[tokio::test]
async fn test_list_posts_single_by_postid() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_1.to_string();

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(postid: "{postid}", limit: 1) {{
                meta {{ ResponseCode }}
                counter
                affectedRows {{ id isliked isviewed }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["counter"].as_i64().unwrap(), 1);
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_list_posts_invalid_postid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(postid: "not-a-uuid", limit: 1) {
                meta { ResponseCode }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "30209");
}

#[tokio::test]
async fn test_list_posts_nonexistent_postid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(postid: "99999999-9999-4999-a999-999999999999", limit: 1) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "21518");
}

#[tokio::test]
async fn test_guest_list_post() {
    let state = default_shared_state();
    let postid = SEED_POST_1.to_string();

    let res = graphql_stateful(
        &state,
        &format!(
            r#"
        query {{
            guestListPost(postid: "{postid}") {{
                meta {{ status ResponseCode }}
                counter
                affectedRows {{
                    id title contenttype
                    isliked isviewed isdisliked issaved
                    user {{ id username slug }}
                }}
            }}
        }}
    "#
        ),
    )
    .await;

    let data = &res["data"]["guestListPost"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["counter"].as_i64().unwrap(), 1);
    let post = &data["affectedRows"][0];
    assert_eq!(post["isliked"], false);
    assert_eq!(post["isviewed"], false);
    assert_eq!(post["isdisliked"], false);
    assert_eq!(post["issaved"], false);
}

#[tokio::test]
async fn test_guest_list_post_not_found() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            guestListPost(postid: "99999999-9999-4999-a999-999999999999") {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(
        res["data"]["guestListPost"]["meta"]["ResponseCode"],
        "31510"
    );
}

#[tokio::test]
async fn test_guest_list_post_invalid_uuid() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            guestListPost(postid: "not-a-uuid") {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(
        res["data"]["guestListPost"]["meta"]["ResponseCode"],
        "30209"
    );
}

#[tokio::test]
async fn test_list_user_posts() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let userid = SEED_USER_VERIFIED.to_string();

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(userid: "{userid}", sortBy: NEWEST, offset: 0, limit: 20) {{
                meta {{ ResponseCode }}
                affectedRows {{ user {{ id }} }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    let rows = data["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["user"]["id"], userid);
    }
}

#[tokio::test]
async fn test_list_posts_without_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 10) {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_create_post_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "My new post",
                contenttype: text,
                mediadescription: "Test description"
            }) {
                meta { ResponseCode }
                affectedRows { id contenttype title }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["createPost"];
    assert_eq!(data["meta"]["ResponseCode"], "11508");
    assert!(data["affectedRows"]["id"].is_string());
    assert_eq!(data["affectedRows"]["contenttype"], "text");
    assert_eq!(data["affectedRows"]["title"], "My new post");
}

#[tokio::test]
async fn test_create_post_empty_title() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "",
                contenttype: text
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30210");
    assert!(res["data"]["createPost"]["affectedRows"].is_null());
}

#[tokio::test]
async fn test_create_post_title_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let long_title = "a".repeat(64);

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            createPost(action: POST, input: {{
                title: "{long_title}",
                contenttype: text
            }}) {{
                meta {{ ResponseCode }}
                affectedRows {{ id }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30210");
}

#[tokio::test]
async fn test_create_post_invalid_tag_format() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Tag test",
                contenttype: text,
                tags: ["a"]
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30262");
}

#[tokio::test]
async fn test_create_post_too_many_tags() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Tag test",
                contenttype: text,
                tags: ["tag1","tag2","tag3","tag4","tag5","tag6","tag7","tag8","tag9","tag10","tag11"]
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30262");
}

#[tokio::test]
async fn test_create_post_description_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let long_desc = "a".repeat(501);

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            createPost(action: POST, input: {{
                title: "Desc test",
                contenttype: text,
                mediadescription: "{long_desc}"
            }}) {{
                meta {{ ResponseCode }}
                affectedRows {{ id }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30263");
}

#[tokio::test]
async fn test_created_post_appears_in_list() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Create a post
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Unique findable post",
                contenttype: text,
                mediadescription: "For search test"
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;
    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "11508");

    // Search for it
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(title: "Unique findable", sortBy: NEWEST, offset: 0, limit: 10) {
                affectedRows { title }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(rows.iter().any(|r| r["title"] == "Unique findable post"));
}

#[tokio::test]
async fn test_create_post_without_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Unauth post",
                contenttype: text
            }) {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_list_posts_followed_filter() {
    let state = default_shared_state();
    let token_alice = login_alice(&state).await;

    // Alice follows verified_user
    graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleFollow(userid: "{}") {{
                meta {{ ResponseCode }}
            }}
        }}
    "#,
            SEED_USER_VERIFIED
        ),
        &token_alice,
    )
    .await;

    // Now list posts with FOLLOWED filter as alice
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(filterBy: [FOLLOWED], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { user { id } }
            }
        }
    "#,
        &token_alice,
    )
    .await;

    let data = &res["data"]["listPosts"];
    let rows = data["affectedRows"].as_array().unwrap();
    if !rows.is_empty() {
        for row in rows {
            let author_id = row["user"]["id"].as_str().unwrap();
            assert!(
                author_id == SEED_USER_BOB.to_string()
                    || author_id == SEED_USER_VERIFIED.to_string()
            );
        }
    }
}

#[tokio::test]
async fn test_list_advertisement_posts() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listAdvertisementPosts(offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows {
                    post { id title }
                    advertisement { advertisementid advertisementtype }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listAdvertisementPosts"];
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    assert!(code == "22002" || code == "12002");
}
