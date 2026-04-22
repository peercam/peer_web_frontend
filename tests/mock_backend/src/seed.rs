use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use uuid::{Uuid, uuid};

use crate::state::{
    AdvertisementRecord, COMMENT_GEM_RETURN, ChatMessageRecord, ChatRecord, CommentRecord,
    ContentVisibilityState, GemRecord, LIKE_GEM_RETURN, MockState, ModerationTicketRecord,
    PostRecord, SYSTEM_BURN_ACCOUNT, SYSTEM_MINT_ACCOUNT, SYSTEM_PEER_ACCOUNT, SYSTEM_SHOP_ACCOUNT,
    ShopDeliveryRecord, ShopOrderRecord, TransactionFeesRecord, TransactionRecord, User,
    UserPreferencesState, VIEW_GEM_RETURN,
};
use crate::types::ad::AdvertisementType;
use crate::types::wallet::TransactionCategory;

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

// --- Phase 4 seed comment UUIDs ---
pub const SEED_COMMENT_1: Uuid = uuid!("20000000-0000-4000-a000-000000000001");
pub const SEED_COMMENT_2: Uuid = uuid!("20000000-0000-4000-a000-000000000002");
pub const SEED_COMMENT_3: Uuid = uuid!("20000000-0000-4000-a000-000000000003");
pub const SEED_COMMENT_4: Uuid = uuid!("20000000-0000-4000-a000-000000000004");
pub const SEED_COMMENT_5: Uuid = uuid!("20000000-0000-4000-a000-000000000005");
pub const SEED_COMMENT_6: Uuid = uuid!("20000000-0000-4000-a000-000000000006");

// --- Phase 4 seed chat UUIDs ---
pub const SEED_CHAT_PRIVATE: Uuid = uuid!("30000000-0000-4000-a000-000000000001");
pub const SEED_CHAT_GROUP: Uuid = uuid!("30000000-0000-4000-a000-000000000002");

// --- Phase 4 seed chat message UUIDs ---
pub const SEED_MSG_1: Uuid = uuid!("40000000-0000-4000-a000-000000000001");
pub const SEED_MSG_2: Uuid = uuid!("40000000-0000-4000-a000-000000000002");
pub const SEED_MSG_3: Uuid = uuid!("40000000-0000-4000-a000-000000000003");
pub const SEED_MSG_4: Uuid = uuid!("40000000-0000-4000-a000-000000000004");
pub const SEED_MSG_5: Uuid = uuid!("40000000-0000-4000-a000-000000000005");

// --- Phase 5 seed transaction UUIDs ---
pub const SEED_TX_1: Uuid = uuid!("50000000-0000-4000-a000-000000000001");
pub const SEED_TX_2: Uuid = uuid!("50000000-0000-4000-a000-000000000002");
pub const SEED_TX_3: Uuid = uuid!("50000000-0000-4000-a000-000000000003");
pub const SEED_OP_1: Uuid = uuid!("50000000-0000-4000-a000-000000000101");
pub const SEED_OP_2: Uuid = uuid!("50000000-0000-4000-a000-000000000102");
pub const SEED_OP_3: Uuid = uuid!("50000000-0000-4000-a000-000000000103");

// --- Phase 5 seed advertisement UUID ---
pub const SEED_AD_1: Uuid = uuid!("60000000-0000-4000-a000-000000000001");

// --- Phase 5 seed shop order UUID ---
pub const SEED_SHOP_ORDER_1: Uuid = uuid!("70000000-0000-4000-a000-000000000001");
pub const SEED_SHOP_TX_1: Uuid = uuid!("70000000-0000-4000-a000-000000000002");

/// Peer Shop operator account.
///
/// This UUID is the single source of truth shared with the client constant
/// `PEER_SHOP_ID` (see `peer-web/src/utils/constants.rs`). The shop operator
/// is allowed to view delivery details for any shop order, mirroring the
/// legacy `PEER_SHOP_ID` UI gate in `js/global.js`.
///
/// Login credentials: `shop@peer.com` / `ShopPass123`.
pub const SEED_USER_SHOP: Uuid = uuid!("292bebb1-0951-47e8-ac8a-759138a2e4a9");

// --- Phase 6 seed admin/moderator user UUIDs ---
pub const SEED_USER_ADMIN: Uuid = uuid!("ad000000-0000-4000-a000-000000000001");
pub const SEED_USER_MODERATOR: Uuid = uuid!("ad000000-0000-4000-a000-000000000002");

// --- Phase 6 seed moderation ticket UUIDs ---
pub const SEED_MOD_TICKET_POST: Uuid = uuid!("ad000000-0000-4000-a000-00000000e001");
pub const SEED_MOD_TICKET_COMMENT: Uuid = uuid!("ad000000-0000-4000-a000-00000000e002");
pub const SEED_MOD_TICKET_USER: Uuid = uuid!("ad000000-0000-4000-a000-00000000e003");

/// Mint initial balance constant.
pub const MINT_INITIAL_BALANCE: Decimal = Decimal::from_parts(50_000_000, 0, 0, false, 1); // 5_000_000.0

/// Default token balance for seeded users.
pub const DEFAULT_USER_BALANCE: Decimal = Decimal::from_parts(10000, 0, 0, false, 1); // 1000.0

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

pub mod credentials_phase6 {
    pub const ADMIN_EMAIL: &str = "admin@peerapp.de";
    pub const ADMIN_PASSWORD: &str = "Admin1234";
    pub const ADMIN_USERNAME: &str = "admin";

    pub const MOD_EMAIL: &str = "mod@peerapp.de";
    pub const MOD_PASSWORD: &str = "Mod1234";
    pub const MOD_USERNAME: &str = "moderator";
}

/// Peer Shop operator credentials (UUID matches client `PEER_SHOP_ID`).
pub mod credentials_shop {
    pub const SHOP_EMAIL: &str = "shop@peer.com";
    pub const SHOP_PASSWORD: &str = "ShopPass123";
    pub const SHOP_USERNAME: &str = "peer_shop";
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
                ip: Some("192.168.1.2".to_string()),
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
                ip: Some("192.168.1.3".to_string()),
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
                ip: Some("192.168.1.4".to_string()),
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
                ip: Some("192.168.1.5".to_string()),
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
                ip: Some("192.168.1.6".to_string()),
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
                ip: Some("192.168.1.7".to_string()),
                created_at: "2025-04-01T10:00:00Z".to_string(),
                updated_at: "2025-04-01T10:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_DAVE, DAVE_PASSWORD.to_string());
        registered_emails.insert(DAVE_EMAIL.to_string());
        verified_users.insert(SEED_USER_DAVE);

        // ====================================================================
        // Phase 6 seed admin & moderator users
        // ====================================================================
        use credentials_phase6::*;

        // Admin user
        users.insert(
            SEED_USER_ADMIN,
            User {
                uid: SEED_USER_ADMIN,
                email: ADMIN_EMAIL.to_string(),
                username: ADMIN_USERNAME.to_string(),
                slug: "admin".to_string(),
                slug_num: 10007,
                role: 16,
                status: 0,
                img: None,
                biography: None,
                visibility_status: ContentVisibilityState::Normal,
                ip: Some("192.168.1.1".to_string()),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_ADMIN, ADMIN_PASSWORD.to_string());
        registered_emails.insert(ADMIN_EMAIL.to_string());
        verified_users.insert(SEED_USER_ADMIN);

        // Moderator user
        users.insert(
            SEED_USER_MODERATOR,
            User {
                uid: SEED_USER_MODERATOR,
                email: MOD_EMAIL.to_string(),
                username: MOD_USERNAME.to_string(),
                slug: "moderator".to_string(),
                slug_num: 10008,
                role: 256,
                status: 0,
                img: None,
                biography: None,
                visibility_status: ContentVisibilityState::Normal,
                ip: Some("192.168.1.2".to_string()),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        );
        user_passwords.insert(SEED_USER_MODERATOR, MOD_PASSWORD.to_string());
        registered_emails.insert(MOD_EMAIL.to_string());
        verified_users.insert(SEED_USER_MODERATOR);

        // Peer Shop operator (UUID matches client `PEER_SHOP_ID`).
        {
            use credentials_shop::*;
            users.insert(
                SEED_USER_SHOP,
                User {
                    uid: SEED_USER_SHOP,
                    email: SHOP_EMAIL.to_string(),
                    username: SHOP_USERNAME.to_string(),
                    slug: "peer_shop".to_string(),
                    slug_num: 10009,
                    role: 0,
                    status: 0,
                    img: None,
                    biography: None,
                    visibility_status: ContentVisibilityState::Normal,
                    ip: Some("192.168.1.10".to_string()),
                    created_at: "2025-01-01T00:00:00Z".to_string(),
                    updated_at: "2025-01-01T00:00:00Z".to_string(),
                },
            );
            user_passwords.insert(SEED_USER_SHOP, SHOP_PASSWORD.to_string());
            registered_emails.insert(SHOP_EMAIL.to_string());
            verified_users.insert(SEED_USER_SHOP);
        }

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
            SEED_USER_ADMIN,
            SEED_USER_MODERATOR,
            SEED_USER_SHOP,
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
            comments: seed_comments(SEED_USER_VERIFIED, SEED_USER_ALICE),
            comment_likes: seed_comment_likes(SEED_USER_VERIFIED, SEED_USER_ALICE),
            comment_reports: HashSet::new(),
            chats: seed_chats(SEED_USER_VERIFIED, SEED_USER_ALICE, SEED_USER_BOB),
            chat_messages: seed_chat_messages(SEED_USER_VERIFIED, SEED_USER_ALICE),
            chat_last_read_at: HashMap::new(),
            wallets: seed_wallets(&[
                SEED_USER_VERIFIED,
                SEED_USER_UNVERIFIED,
                SEED_USER_ALICE,
                SEED_USER_BOB,
                SEED_USER_CAROL,
                SEED_USER_DAVE,
                SEED_USER_ADMIN,
                SEED_USER_MODERATOR,
                SEED_USER_SHOP,
            ]),
            transactions: seed_transactions(SEED_USER_VERIFIED, SEED_USER_ALICE),
            daily_actions_used: HashMap::new(),
            gems: seed_gems(SEED_USER_VERIFIED, SEED_USER_ALICE),
            minted_dates: HashSet::new(),
            shop_orders: seed_shop_orders(SEED_USER_ALICE),
            moderation_tickets: seed_moderation_tickets(),
            content_visibility: seed_content_visibility(),
            alpha_minted: false,
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
        id: SEED_AD_1,
        post_id: SEED_POST_8,
        advertiser_id: SEED_USER_VERIFIED,
        ad_type: AdvertisementType::Basic,
        start_date: "2025-04-01".into(),
        end_date: "2025-05-01".into(),
        token_cost: Decimal::from_parts(1500, 0, 0, false, 1), // 150.0
        created_at: "2025-04-01T08:00:00Z".into(),
    }]
}

// ============================================================================
// Phase 4: Seed Comment & Chat Data
// ============================================================================

fn seed_comments(verified_user: Uuid, user2: Uuid) -> Vec<CommentRecord> {
    vec![
        // Top-level comment on post 1 by user2
        CommentRecord {
            id: SEED_COMMENT_1,
            author_id: user2,
            post_id: SEED_POST_1,
            parent_id: None,
            content: "Great photo! Love the colors.".into(),
            created_at: "2025-04-01T12:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Top-level comment on post 1 by verified_user
        CommentRecord {
            id: SEED_COMMENT_2,
            author_id: verified_user,
            post_id: SEED_POST_1,
            parent_id: None,
            content: "Thanks for the kind words!".into(),
            created_at: "2025-04-01T13:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Top-level comment on post 3 by verified_user
        CommentRecord {
            id: SEED_COMMENT_3,
            author_id: verified_user,
            post_id: SEED_POST_3,
            parent_id: None,
            content: "These Rust tips are really helpful.".into(),
            created_at: "2025-04-03T15:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Top-level comment on post 5 by user2
        CommentRecord {
            id: SEED_COMMENT_4,
            author_id: user2,
            post_id: SEED_POST_5,
            parent_id: None,
            content: "Amazing tutorial, well explained!".into(),
            created_at: "2025-04-05T17:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Reply to comment 1 by verified_user
        CommentRecord {
            id: SEED_COMMENT_5,
            author_id: verified_user,
            post_id: SEED_POST_1,
            parent_id: Some(SEED_COMMENT_1),
            content: "Glad you liked it!".into(),
            created_at: "2025-04-01T14:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Reply to comment 3 by user2
        CommentRecord {
            id: SEED_COMMENT_6,
            author_id: user2,
            post_id: SEED_POST_3,
            parent_id: Some(SEED_COMMENT_3),
            content: "You're welcome, check out part 2 as well!".into(),
            created_at: "2025-04-03T16:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
    ]
}

fn seed_comment_likes(verified_user: Uuid, user2: Uuid) -> HashSet<(Uuid, Uuid)> {
    HashSet::from([
        (verified_user, SEED_COMMENT_4), // verified user liked user2's comment
        (user2, SEED_COMMENT_3),         // user2 liked verified user's comment
    ])
}

fn seed_chats(verified_user: Uuid, user2: Uuid, user3: Uuid) -> Vec<ChatRecord> {
    vec![
        // Private 1:1 chat between verified_user and user2
        ChatRecord {
            id: SEED_CHAT_PRIVATE,
            name: None,
            image: None,
            created_at: "2025-04-01T10:00:00Z".into(),
            updated_at: "2025-04-02T14:30:00Z".into(),
            participant_ids: vec![verified_user, user2],
        },
        // Group chat with 3 participants
        ChatRecord {
            id: SEED_CHAT_GROUP,
            name: Some("Rust Developers".into()),
            image: None,
            created_at: "2025-04-03T09:00:00Z".into(),
            updated_at: "2025-04-04T11:00:00Z".into(),
            participant_ids: vec![verified_user, user2, user3],
        },
    ]
}

fn seed_chat_messages(verified_user: Uuid, user2: Uuid) -> Vec<ChatMessageRecord> {
    vec![
        // Private chat messages
        ChatMessageRecord {
            id: SEED_MSG_1,
            sender_id: verified_user,
            chat_id: SEED_CHAT_PRIVATE,
            content: "Hey, how's the project going?".into(),
            created_at: "2025-04-01T10:05:00Z".into(),
        },
        ChatMessageRecord {
            id: SEED_MSG_2,
            sender_id: user2,
            chat_id: SEED_CHAT_PRIVATE,
            content: "Going well! Just deployed the new feature.".into(),
            created_at: "2025-04-02T14:00:00Z".into(),
        },
        ChatMessageRecord {
            id: SEED_MSG_3,
            sender_id: verified_user,
            chat_id: SEED_CHAT_PRIVATE,
            content: "Awesome, I'll check it out.".into(),
            created_at: "2025-04-02T14:30:00Z".into(),
        },
        // Group chat messages
        ChatMessageRecord {
            id: SEED_MSG_4,
            sender_id: verified_user,
            chat_id: SEED_CHAT_GROUP,
            content: "Welcome to the Rust devs group!".into(),
            created_at: "2025-04-03T09:05:00Z".into(),
        },
        ChatMessageRecord {
            id: SEED_MSG_5,
            sender_id: user2,
            chat_id: SEED_CHAT_GROUP,
            content: "Thanks for the invite. What are we working on first?".into(),
            created_at: "2025-04-04T11:00:00Z".into(),
        },
    ]
}

// ============================================================================
// Phase 5: Seed Economy Data
// ============================================================================

fn seed_wallets(users: &[Uuid]) -> HashMap<Uuid, Decimal> {
    let mut wallets = HashMap::new();
    for &uid in users {
        if uid == SEED_USER_ADMIN {
            wallets.insert(uid, Decimal::from_parts(100_000, 0, 0, false, 1)); // 10000.0
        } else if uid == SEED_USER_MODERATOR {
            wallets.insert(uid, Decimal::from_parts(50_000, 0, 0, false, 1)); // 5000.0
        } else {
            wallets.insert(uid, DEFAULT_USER_BALANCE);
        }
    }
    wallets.insert(SYSTEM_MINT_ACCOUNT, MINT_INITIAL_BALANCE);
    wallets.insert(SYSTEM_PEER_ACCOUNT, Decimal::ZERO);
    wallets.insert(SYSTEM_BURN_ACCOUNT, Decimal::ZERO);
    wallets.insert(SYSTEM_SHOP_ACCOUNT, Decimal::ZERO);
    wallets
}

fn seed_transactions(user1: Uuid, user2: Uuid) -> Vec<TransactionRecord> {
    vec![
        // P2P transfer: user1 sent 50 tokens to user2
        TransactionRecord {
            id: SEED_TX_1,
            operation_id: SEED_OP_1,
            category: Some(TransactionCategory::P2pTransfer),
            transaction_type: "CREDIT".into(),
            sender_id: user1,
            recipient_id: user2,
            token_amount: Decimal::from_parts(500, 0, 0, false, 1), // 50.0
            net_token_amount: Decimal::from_parts(500, 0, 0, false, 1),
            message: Some("Great post!".into()),
            fees: Some(TransactionFeesRecord {
                total: Decimal::from_parts(20, 0, 0, false, 1), // 2.0
                burn: Decimal::from_parts(5, 0, 0, false, 1),   // 0.5
                peer: Decimal::from_parts(10, 0, 0, false, 1),  // 1.0
                inviter: Some(Decimal::from_parts(5, 0, 0, false, 1)), // 0.5
            }),
            created_at: "2025-04-10T10:00:00Z".into(),
        },
        // Like payment: user1 paid 3 tokens for like
        TransactionRecord {
            id: SEED_TX_2,
            operation_id: SEED_OP_2,
            category: Some(TransactionCategory::Like),
            transaction_type: "DEBIT".into(),
            sender_id: user1,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: Decimal::from_parts(30, 0, 0, false, 1), // 3.0
            net_token_amount: Decimal::from_parts(30, 0, 0, false, 1),
            message: None,
            fees: None,
            created_at: "2025-04-11T14:00:00Z".into(),
        },
        // Post creation payment: user2 paid 20 tokens for post
        TransactionRecord {
            id: SEED_TX_3,
            operation_id: SEED_OP_3,
            category: Some(TransactionCategory::PostCreate),
            transaction_type: "DEBIT".into(),
            sender_id: user2,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: Decimal::from_parts(200, 0, 0, false, 1), // 20.0
            net_token_amount: Decimal::from_parts(200, 0, 0, false, 1),
            message: None,
            fees: None,
            created_at: "2025-04-12T09:00:00Z".into(),
        },
        // Shop purchase: user2 (Alice) bought a shop item; recipient is the
        // Peer Shop operator account so the shop user sees this row in their
        // wallet history and can expand it to view delivery details. The id
        // matches the seeded ShopOrderRecord (`SEED_SHOP_TX_1`) so the lazy
        // `shopOrderDetails` lookup resolves to the seeded order.
        TransactionRecord {
            id: SEED_SHOP_TX_1,
            operation_id: uuid!("70000000-0000-4000-a000-000000000102"),
            category: Some(TransactionCategory::ShopPurchase),
            transaction_type: "DEBIT".into(),
            sender_id: user2,
            recipient_id: SEED_USER_SHOP,
            token_amount: Decimal::from_parts(500, 0, 0, false, 1), // 50.0
            net_token_amount: Decimal::from_parts(500, 0, 0, false, 1),
            message: Some("Shop purchase: peer-tshirt-001".into()),
            fees: None,
            created_at: "2025-04-13T15:00:00Z".into(),
        },
    ]
}

fn seed_gems(user1: Uuid, user2: Uuid) -> Vec<GemRecord> {
    vec![
        GemRecord {
            user_id: user1,
            post_id: SEED_POST_1,
            from_user_id: user2,
            gems: LIKE_GEM_RETURN,
            action: "like".into(),
            created_at: "2025-04-11T14:00:00Z".into(),
        },
        GemRecord {
            user_id: user1,
            post_id: SEED_POST_1,
            from_user_id: user2,
            gems: VIEW_GEM_RETURN,
            action: "view".into(),
            created_at: "2025-04-11T13:00:00Z".into(),
        },
        GemRecord {
            user_id: user2,
            post_id: SEED_POST_3,
            from_user_id: user1,
            gems: COMMENT_GEM_RETURN,
            action: "comment".into(),
            created_at: "2025-04-12T10:00:00Z".into(),
        },
    ]
}

fn seed_shop_orders(user2: Uuid) -> Vec<ShopOrderRecord> {
    vec![ShopOrderRecord {
        id: SEED_SHOP_ORDER_1,
        transaction_id: SEED_SHOP_TX_1,
        shop_item_id: "peer-tshirt-001".into(),
        buyer_id: user2,
        token_amount: Decimal::from_parts(500, 0, 0, false, 1), // 50.0
        item_specs: Some("L".into()),
        delivery: ShopDeliveryRecord {
            name: "Test User".into(),
            email: "test@example.com".into(),
            addressline1: "Musterstraße 42".into(),
            addressline2: None,
            city: "Berlin".into(),
            zipcode: "10115".into(),
            country: "GERMANY".into(),
        },
        created_at: "2025-04-13T15:00:00Z".into(),
    }]
}

// ============================================================================
// Phase 6: Seed Moderation Data
// ============================================================================

fn seed_moderation_tickets() -> Vec<ModerationTicketRecord> {
    vec![
        // Post ticket: waiting_for_review, reported by verified user
        ModerationTicketRecord {
            id: SEED_MOD_TICKET_POST,
            target_content_id: SEED_POST_1,
            target_type: "post".into(),
            reporter_ids: vec![SEED_USER_ALICE],
            reports_count: 1,
            status: "waiting_for_review".into(),
            moderated_by: None,
            created_at: "2025-04-10T10:00:00Z".into(),
        },
        // Comment ticket: waiting_for_review, reported by bob
        ModerationTicketRecord {
            id: SEED_MOD_TICKET_COMMENT,
            target_content_id: SEED_COMMENT_1,
            target_type: "comment".into(),
            reporter_ids: vec![SEED_USER_BOB],
            reports_count: 1,
            status: "waiting_for_review".into(),
            moderated_by: None,
            created_at: "2025-04-10T11:00:00Z".into(),
        },
        // User ticket: hidden, moderated by moderator, reported by alice and bob
        ModerationTicketRecord {
            id: SEED_MOD_TICKET_USER,
            target_content_id: SEED_USER_DAVE,
            target_type: "user".into(),
            reporter_ids: vec![SEED_USER_ALICE, SEED_USER_BOB],
            reports_count: 2,
            status: "hidden".into(),
            moderated_by: Some(SEED_USER_MODERATOR),
            created_at: "2025-04-09T08:00:00Z".into(),
        },
    ]
}

fn seed_content_visibility() -> HashMap<Uuid, String> {
    let mut map = HashMap::new();
    // Dave is hidden (matching user ticket 3)
    map.insert(SEED_USER_DAVE, "HIDDEN".into());
    map
}
