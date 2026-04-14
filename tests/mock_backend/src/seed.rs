use std::collections::{HashMap, HashSet};
use uuid::{Uuid, uuid};

use crate::state::{
    AdvertisementRecord, ContentVisibilityState, MockState, PostRecord, User, UserPreferencesState,
};

/// Primary test referral — matches existing Node.js mock
pub const REFERRAL_PRIMARY: Uuid = uuid!("85d5f836-b1f5-4c4e-9381-1b058e13df93");
/// Secondary test referral
pub const REFERRAL_SECONDARY: Uuid = uuid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

// --- Phase 1 seed users ---
pub const SEED_USER_VERIFIED: Uuid = uuid!("00000000-0000-4000-a000-000000000001");
pub const SEED_USER_UNVERIFIED: Uuid = uuid!("00000000-0000-4000-a000-000000000002");

// --- Phase 2 seed users ---
pub const SEED_USER_ALICE: Uuid = uuid!("00000000-0000-4000-a000-000000000003");
pub const SEED_USER_BOB: Uuid = uuid!("00000000-0000-4000-a000-000000000004");
pub const SEED_USER_CAROL: Uuid = uuid!("00000000-0000-4000-a000-000000000005");
pub const SEED_USER_DAVE: Uuid = uuid!("00000000-0000-4000-a000-000000000006");

// --- Phase 3 seed post UUIDs ---
pub const SEED_POST_1: Uuid = uuid!("10000000-0000-4000-a000-000000000001");
pub const SEED_POST_2: Uuid = uuid!("10000000-0000-4000-a000-000000000002");
pub const SEED_POST_3: Uuid = uuid!("10000000-0000-4000-a000-000000000003");
pub const SEED_POST_4: Uuid = uuid!("10000000-0000-4000-a000-000000000004");
pub const SEED_POST_5: Uuid = uuid!("10000000-0000-4000-a000-000000000005");
pub const SEED_POST_6: Uuid = uuid!("10000000-0000-4000-a000-000000000006");
pub const SEED_POST_7: Uuid = uuid!("10000000-0000-4000-a000-000000000007");
pub const SEED_POST_8: Uuid = uuid!("10000000-0000-4000-a000-000000000008");

/// Pre-known credentials for seeded test users
pub mod credentials {
    /// Verified user: test@peer.com / TestPass123
    pub const VERIFIED_EMAIL: &str = "test@peer.com";
    pub const VERIFIED_PASSWORD: &str = "TestPass123";
    pub const VERIFIED_USERNAME: &str = "peerTester";

    /// Unverified user: unverified@peer.com / TestPass456
    pub const UNVERIFIED_EMAIL: &str = "unverified@peer.com";
    pub const UNVERIFIED_PASSWORD: &str = "TestPass456";
    pub const UNVERIFIED_USERNAME: &str = "newSignup";
}

pub mod credentials_phase2 {
    pub const ALICE_EMAIL: &str = "alice@peer.com";
    pub const ALICE_PASSWORD: &str = "AlicePass123";
    pub const ALICE_USERNAME: &str = "alice_peer";

    pub const BOB_EMAIL: &str = "bob@peer.com";
    pub const BOB_PASSWORD: &str = "BobPass123";
    pub const BOB_USERNAME: &str = "bob_peer";

    pub const CAROL_EMAIL: &str = "carol@peer.com";
    pub const CAROL_PASSWORD: &str = "CarolPass123";
    pub const CAROL_USERNAME: &str = "carol_peer";

    pub const DAVE_EMAIL: &str = "dave@peer.com";
    pub const DAVE_PASSWORD: &str = "DavePass123";
    pub const DAVE_USERNAME: &str = "dave_peer";
}

impl Default for MockState {
    fn default() -> Self {
        use credentials::*;
        use credentials_phase2::*;

        let mut users = HashMap::new();
        let mut user_passwords = HashMap::new();
        let mut registered_emails = HashSet::new();
        let mut verified_users = HashSet::new();
        let mut follows = HashSet::new();
        let mut blocks = HashSet::new();
        let mut preferences = HashMap::new();
        let mut referral_invitations = HashMap::new();

        // ====================================================================
        // Phase 1 seed users (updated with new fields)
        // ====================================================================

        users.insert(
            SEED_USER_VERIFIED,
            User {
                uid: SEED_USER_VERIFIED,
                email: VERIFIED_EMAIL.to_string(),
                username: VERIFIED_USERNAME.to_string(),
                slug: "peertester".to_string(),
                slug_num: 10001,
                role: 0,
                status: 0,
                img: None,
                biography: None,
                visibility_status: ContentVisibilityState::Normal,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_VERIFIED, VERIFIED_PASSWORD.to_string());
        registered_emails.insert(VERIFIED_EMAIL.to_string());
        verified_users.insert(SEED_USER_VERIFIED);

        users.insert(
            SEED_USER_UNVERIFIED,
            User {
                uid: SEED_USER_UNVERIFIED,
                email: UNVERIFIED_EMAIL.to_string(),
                username: UNVERIFIED_USERNAME.to_string(),
                slug: "newsignup".to_string(),
                slug_num: 10002,
                role: 0,
                status: 0,
                img: None,
                biography: None,
                visibility_status: ContentVisibilityState::Normal,
                created_at: "2025-01-02T00:00:00Z".to_string(),
                updated_at: "2025-01-02T00:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_UNVERIFIED, UNVERIFIED_PASSWORD.to_string());
        registered_emails.insert(UNVERIFIED_EMAIL.to_string());

        // ====================================================================
        // Phase 2 seed users
        // ====================================================================

        // Alice (verified, has bio and avatar)
        users.insert(
            SEED_USER_ALICE,
            User {
                uid: SEED_USER_ALICE,
                email: ALICE_EMAIL.to_string(),
                username: ALICE_USERNAME.to_string(),
                slug: "alice_peer".to_string(),
                slug_num: 10003,
                role: 0,
                status: 0,
                img: Some("https://via.placeholder.com/96/alice".to_string()),
                biography: Some("data:text/plain;base64,SGVsbG8gSSdtIEFsaWNl".to_string()),
                visibility_status: ContentVisibilityState::Normal,
                created_at: "2025-01-15T10:00:00Z".to_string(),
                updated_at: "2025-06-01T12:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_ALICE, ALICE_PASSWORD.to_string());
        registered_emails.insert(ALICE_EMAIL.to_string());
        verified_users.insert(SEED_USER_ALICE);

        // Bob (verified, has avatar, no bio)
        users.insert(
            SEED_USER_BOB,
            User {
                uid: SEED_USER_BOB,
                email: BOB_EMAIL.to_string(),
                username: BOB_USERNAME.to_string(),
                slug: "bob_peer".to_string(),
                slug_num: 10004,
                role: 0,
                status: 0,
                img: Some("https://via.placeholder.com/96/bob".to_string()),
                biography: None,
                visibility_status: ContentVisibilityState::Normal,
                created_at: "2025-02-01T10:00:00Z".to_string(),
                updated_at: "2025-05-15T08:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_BOB, BOB_PASSWORD.to_string());
        registered_emails.insert(BOB_EMAIL.to_string());
        verified_users.insert(SEED_USER_BOB);

        // Carol (verified, no avatar, has bio)
        users.insert(
            SEED_USER_CAROL,
            User {
                uid: SEED_USER_CAROL,
                email: CAROL_EMAIL.to_string(),
                username: CAROL_USERNAME.to_string(),
                slug: "carol_peer".to_string(),
                slug_num: 10005,
                role: 0,
                status: 0,
                img: None,
                biography: Some("data:text/plain;base64,Q2Fyb2wgaGVyZQ==".to_string()),
                visibility_status: ContentVisibilityState::Normal,
                created_at: "2025-03-01T10:00:00Z".to_string(),
                updated_at: "2025-04-01T10:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_CAROL, CAROL_PASSWORD.to_string());
        registered_emails.insert(CAROL_EMAIL.to_string());
        verified_users.insert(SEED_USER_CAROL);

        // Dave (verified, no avatar, no bio)
        users.insert(
            SEED_USER_DAVE,
            User {
                uid: SEED_USER_DAVE,
                email: DAVE_EMAIL.to_string(),
                username: DAVE_USERNAME.to_string(),
                slug: "dave_peer".to_string(),
                slug_num: 10006,
                role: 0,
                status: 0,
                img: None,
                biography: None,
                visibility_status: ContentVisibilityState::Normal,
                created_at: "2025-04-01T10:00:00Z".to_string(),
                updated_at: "2025-04-01T10:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_DAVE, DAVE_PASSWORD.to_string());
        registered_emails.insert(DAVE_EMAIL.to_string());
        verified_users.insert(SEED_USER_DAVE);

        // ====================================================================
        // Phase 2 seed relationships
        // ====================================================================

        // alice ↔ bob (mutual = friends)
        follows.insert((SEED_USER_ALICE, SEED_USER_BOB));
        follows.insert((SEED_USER_BOB, SEED_USER_ALICE));
        // carol → alice (one-directional)
        follows.insert((SEED_USER_CAROL, SEED_USER_ALICE));

        // carol blocks dave
        blocks.insert((SEED_USER_CAROL, SEED_USER_DAVE));

        // alice was invited by seed_verified user
        referral_invitations.insert(SEED_USER_ALICE, SEED_USER_VERIFIED);

        // Default preferences for all seeded users
        for uid in [
            SEED_USER_VERIFIED,
            SEED_USER_ALICE,
            SEED_USER_BOB,
            SEED_USER_CAROL,
            SEED_USER_DAVE,
        ] {
            preferences.insert(uid, UserPreferencesState::default());
        }

        Self {
            known_referrals: HashSet::from([REFERRAL_PRIMARY, REFERRAL_SECONDARY]),
            registered_emails,
            verified_users,
            users,
            user_passwords,
            access_tokens: HashMap::new(),
            refresh_tokens: HashMap::new(),
            password_reset_tokens: HashMap::new(),
            deleted_users: HashSet::new(),
            contact_messages: Vec::new(),
            follows,
            blocks,
            reports: Vec::new(),
            preferences,
            referral_invitations,
            posts: seed_posts(SEED_USER_VERIFIED, SEED_USER_ALICE),
            post_likes: seed_post_likes(SEED_USER_VERIFIED, SEED_USER_ALICE),
            post_dislikes: HashSet::new(),
            post_saves: HashSet::new(),
            post_views: seed_post_views(SEED_USER_VERIFIED, SEED_USER_ALICE),
            post_shares: HashSet::new(),
            post_reports: HashSet::new(),
            tags: seed_tags(),
            eligibility_tokens: HashMap::new(),
            eligibility_token_status: HashMap::new(),
            uploaded_files: HashMap::new(),
            advertisements: seed_advertisements(),
        }
    }
}

/// Mock referral user data returned on successful verification
pub mod mock_users {
    use crate::types::registration::ReferralUser;
    use async_graphql::ID;

    pub fn referral_user() -> ReferralUser {
        ReferralUser {
            uid: ID::from("usr_mock_001"),
            username: "peerTester".to_string(),
            slug: "peertester".to_string(),
            img: Some("https://via.placeholder.com/96".to_string()),
        }
    }
}

// ============================================================================
// Phase 3: Seed Post Data
// ============================================================================

fn seed_posts(verified_user: Uuid, user2: Uuid) -> Vec<PostRecord> {
    vec![
        PostRecord {
            id: SEED_POST_1,
            author_id: verified_user,
            contenttype: "image".into(),
            title: "My first photo".into(),
            media: "https://mock.peer.com/media/photo1.jpg".into(),
            cover: "https://mock.peer.com/covers/photo1_cover.jpg".into(),
            mediadescription: "A beautiful sunset".into(),
            created_at: "2025-04-01T10:00:00Z".into(),
            tags: vec!["peer".into(), "webdev".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_2,
            author_id: verified_user,
            contenttype: "text".into(),
            title: "Hello Peer Network".into(),
            media: "".into(),
            cover: "".into(),
            mediadescription: "Welcome to my first text post!".into(),
            created_at: "2025-04-02T12:00:00Z".into(),
            tags: vec!["peer".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_3,
            author_id: user2,
            contenttype: "image".into(),
            title: "Rust programming tips".into(),
            media: "https://mock.peer.com/media/rust_tips.png".into(),
            cover: "https://mock.peer.com/covers/rust_tips_cover.png".into(),
            mediadescription: "Helpful Rust patterns".into(),
            created_at: "2025-04-03T14:00:00Z".into(),
            tags: vec!["rust".into(), "tutorial".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_4,
            author_id: user2,
            contenttype: "audio".into(),
            title: "Peer podcast episode 1".into(),
            media: "https://mock.peer.com/media/podcast_ep1.mp3".into(),
            cover: "https://mock.peer.com/covers/podcast_cover.jpg".into(),
            mediadescription: "Our first podcast episode".into(),
            created_at: "2025-04-04T09:00:00Z".into(),
            tags: vec!["peer".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_5,
            author_id: verified_user,
            contenttype: "video".into(),
            title: "Web dev tutorial".into(),
            media: "https://mock.peer.com/media/tutorial.mp4".into(),
            cover: "https://mock.peer.com/covers/tutorial_cover.jpg".into(),
            mediadescription: "Learn web development step by step".into(),
            created_at: "2025-04-05T16:00:00Z".into(),
            tags: vec!["webdev".into(), "tutorial".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_6,
            author_id: verified_user,
            contenttype: "text".into(),
            title: "Thoughts on decentralization".into(),
            media: "".into(),
            cover: "".into(),
            mediadescription: "Why peer-to-peer matters".into(),
            created_at: "2025-04-06T11:00:00Z".into(),
            tags: vec!["peer".into(), "webdev".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_7,
            author_id: user2,
            contenttype: "image".into(),
            title: "Weekend project showcase".into(),
            media:
                "https://mock.peer.com/media/project.jpg,https://mock.peer.com/media/project2.jpg"
                    .into(),
            cover: "https://mock.peer.com/covers/project_cover.jpg".into(),
            mediadescription: "Check out what I built this weekend".into(),
            created_at: "2025-04-07T18:00:00Z".into(),
            tags: vec!["rust".into(), "webdev".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_8,
            author_id: verified_user,
            contenttype: "text".into(),
            title: "Sponsored: Learn Rust".into(),
            media: "".into(),
            cover: "".into(),
            mediadescription: "Sponsored post about learning Rust".into(),
            created_at: "2025-04-08T08:00:00Z".into(),
            tags: vec!["rust".into(), "tutorial".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
    ]
}

fn seed_tags() -> HashSet<String> {
    HashSet::from([
        "rust".into(),
        "webdev".into(),
        "peer".into(),
        "tutorial".into(),
    ])
}

fn seed_post_likes(verified_user: Uuid, user2: Uuid) -> HashSet<(Uuid, Uuid)> {
    HashSet::from([(verified_user, SEED_POST_3), (user2, SEED_POST_1)])
}

fn seed_post_views(verified_user: Uuid, user2: Uuid) -> HashSet<(Uuid, Uuid)> {
    HashSet::from([
        (verified_user, SEED_POST_3),
        (verified_user, SEED_POST_4),
        (user2, SEED_POST_1),
    ])
}

fn seed_advertisements() -> Vec<AdvertisementRecord> {
    vec![AdvertisementRecord {
        id: "ad-001".into(),
        post_id: SEED_POST_8,
        advertisement_type: "STANDARD".into(),
        start_date: "2025-04-01".into(),
        end_date: "2025-05-01".into(),
    }]
}
