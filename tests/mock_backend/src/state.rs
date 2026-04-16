use chrono::Datelike;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::ad::AdvertisementType;
use crate::types::wallet::TransactionCategory;

/// Shared state wrapped in Arc<RwLock<>>
pub type SharedState = Arc<RwLock<MockState>>;

/// Content visibility status (state-level).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContentVisibilityState {
    #[default]
    Normal,
    Hidden,
    Illegal,
}

/// Content filtering severity level (state-level).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContentFilterState {
    #[default]
    Mygrandmalikes,
    Mygrandmahates,
}

/// A mock user record
#[derive(Debug, Clone)]
pub struct User {
    pub uid: Uuid,
    pub email: String,
    pub username: String,
    pub slug: String,
    pub slug_num: i32,
    pub role: u32,   // 0 = user, 16 = admin, 256 = moderator
    pub status: u32, // 0 = active, 6 = deleted
    pub img: Option<String>,
    pub biography: Option<String>,
    pub visibility_status: ContentVisibilityState,
    pub ip: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// User preferences stored in state.
#[derive(Debug, Clone)]
pub struct UserPreferencesState {
    pub content_filtering_severity_level: ContentFilterState,
    pub onboardings_were_shown: Vec<String>,
}

impl Default for UserPreferencesState {
    fn default() -> Self {
        Self {
            content_filtering_severity_level: ContentFilterState::Mygrandmalikes,
            onboardings_were_shown: Vec::new(),
        }
    }
}

/// A user report record.
#[derive(Debug, Clone)]
pub struct UserReport {
    pub reporter: Uuid,
    pub reported: Uuid,
    pub created_at: String,
}

/// Contact message record
#[derive(Debug, Clone)]
pub struct ContactMessage {
    pub name: String,
    pub email: String,
    pub message: String,
}

/// Internal post storage record (not the GraphQL type).
#[derive(Debug, Clone)]
pub struct PostRecord {
    pub id: Uuid,
    pub author_id: Uuid,
    pub contenttype: String,
    pub title: String,
    pub media: String,
    pub cover: String,
    pub mediadescription: String,
    pub created_at: String,
    pub tags: Vec<String>,
    pub visibility_status: String,
    pub uploaded_files: Option<String>,
}

/// Internal advertisement record (Phase 5).
#[derive(Debug, Clone)]
pub struct AdvertisementRecord {
    pub id: Uuid,
    pub post_id: Uuid,
    pub advertiser_id: Uuid,
    pub ad_type: AdvertisementType,
    pub start_date: String,
    pub end_date: String,
    pub token_cost: Decimal,
    pub created_at: String,
}

// ============================================================================
// Phase 5: Economy state records
// ============================================================================

/// System account constants.
pub const SYSTEM_BURN_ACCOUNT: Uuid = Uuid::from_bytes([
    0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0x4e, 0xee, 0xae, 0xee, 0, 0, 0, 0, 0, 1,
]);
pub const SYSTEM_PEER_ACCOUNT: Uuid = Uuid::from_bytes([
    0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0x4e, 0xee, 0xae, 0xee, 0, 0, 0, 0, 0, 2,
]);
pub const SYSTEM_SHOP_ACCOUNT: Uuid = Uuid::from_bytes([
    0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0x4e, 0xee, 0xae, 0xee, 0, 0, 0, 0, 0, 3,
]);
pub const SYSTEM_MINT_ACCOUNT: Uuid = Uuid::from_bytes([
    0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0x4e, 0xee, 0xae, 0xee, 0, 0, 0, 0, 0, 4,
]);

/// Action pricing constants.
pub const POST_PRICE: Decimal = Decimal::from_parts(200, 0, 0, false, 1); // 20.0
pub const LIKE_PRICE: Decimal = Decimal::from_parts(30, 0, 0, false, 1); // 3.0
pub const DISLIKE_PRICE: Decimal = Decimal::from_parts(30, 0, 0, false, 1); // 3.0
pub const COMMENT_PRICE: Decimal = Decimal::from_parts(10, 0, 0, false, 1); // 1.0
pub const AD_BASIC_DAILY_PRICE: Decimal = Decimal::from_parts(500, 0, 0, false, 1); // 50.0
pub const AD_PINNED_PRICE: Decimal = Decimal::from_parts(2000, 0, 0, false, 1); // 200.0

/// Daily free action limits.
pub const FREE_POSTS: u32 = 1;
pub const FREE_LIKES: u32 = 3;
pub const FREE_COMMENTS: u32 = 4;
pub const FREE_DISLIKES: u32 = 0;

/// Gem return rates.
pub const VIEW_GEM_RETURN: f64 = 0.25;
pub const LIKE_GEM_RETURN: f64 = 5.0;
pub const DISLIKE_GEM_RETURN: f64 = -3.0;
pub const COMMENT_GEM_RETURN: f64 = 2.0;

/// Fee rates for transfers.
pub const BURN_FEE_RATE: Decimal = Decimal::from_parts(1, 0, 0, false, 2); // 0.01
pub const PEER_FEE_RATE: Decimal = Decimal::from_parts(2, 0, 0, false, 2); // 0.02
pub const INVITER_FEE_RATE: Decimal = Decimal::from_parts(1, 0, 0, false, 2); // 0.01

/// Internal transaction storage record.
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub category: Option<TransactionCategory>,
    pub transaction_type: String,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub token_amount: Decimal,
    pub net_token_amount: Decimal,
    pub message: Option<String>,
    pub fees: Option<TransactionFeesRecord>,
    pub created_at: String,
}

/// Internal fee record.
#[derive(Debug, Clone)]
pub struct TransactionFeesRecord {
    pub total: Decimal,
    pub burn: Decimal,
    pub peer: Decimal,
    pub inviter: Option<Decimal>,
}

/// Internal gem storage record.
#[derive(Debug, Clone)]
pub struct GemRecord {
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub from_user_id: Uuid,
    pub gems: f64,
    pub action: String,
    pub created_at: String,
}

/// Internal shop order storage record.
#[derive(Debug, Clone)]
pub struct ShopOrderRecord {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub shop_item_id: String,
    pub buyer_id: Uuid,
    pub token_amount: Decimal,
    pub item_specs: Option<String>,
    pub delivery: ShopDeliveryRecord,
    pub created_at: String,
}

/// Internal delivery details record.
#[derive(Debug, Clone)]
pub struct ShopDeliveryRecord {
    pub name: String,
    pub email: String,
    pub addressline1: String,
    pub addressline2: Option<String>,
    pub city: String,
    pub zipcode: String,
    pub country: String,
}

/// A moderation ticket record.
#[derive(Debug, Clone)]
pub struct ModerationTicketRecord {
    pub id: Uuid,
    pub target_content_id: Uuid,
    pub target_type: String,
    pub reporter_ids: Vec<Uuid>,
    pub reports_count: i32,
    pub status: String,
    pub moderated_by: Option<Uuid>,
    pub created_at: String,
}

/// Internal comment storage record (not the GraphQL type).
#[derive(Debug, Clone)]
pub struct CommentRecord {
    pub id: Uuid,
    pub author_id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub created_at: String,
    pub visibility_status: String,
}

/// Internal chat storage record.
#[derive(Debug, Clone)]
pub struct ChatRecord {
    pub id: Uuid,
    pub name: Option<String>,
    pub image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub participant_ids: Vec<Uuid>,
}

/// Internal chat message storage record.
#[derive(Debug, Clone)]
pub struct ChatMessageRecord {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub chat_id: Uuid,
    pub content: String,
    pub created_at: String,
}

/// In-memory mock backend state
#[derive(Debug, Clone)]
pub struct MockState {
    // --- Phase 0 fields ---
    pub known_referrals: HashSet<Uuid>,
    pub registered_emails: HashSet<String>,
    pub verified_users: HashSet<Uuid>,

    // --- Phase 1 fields ---
    pub users: HashMap<Uuid, User>,
    pub user_passwords: HashMap<Uuid, String>,
    pub access_tokens: HashMap<String, Uuid>,
    pub refresh_tokens: HashMap<String, Uuid>,
    pub password_reset_tokens: HashMap<String, Uuid>,
    pub deleted_users: HashSet<Uuid>,
    pub contact_messages: Vec<ContactMessage>,

    // --- Phase 2 fields ---
    pub follows: HashSet<(Uuid, Uuid)>,
    pub blocks: HashSet<(Uuid, Uuid)>,
    pub reports: Vec<UserReport>,
    pub preferences: HashMap<Uuid, UserPreferencesState>,
    pub referral_invitations: HashMap<Uuid, Uuid>,

    // --- Phase 3 fields ---
    pub posts: Vec<PostRecord>,
    pub post_likes: HashSet<(Uuid, Uuid)>,
    pub post_dislikes: HashSet<(Uuid, Uuid)>,
    pub post_saves: HashSet<(Uuid, Uuid)>,
    pub post_views: HashSet<(Uuid, Uuid)>,
    pub post_shares: HashSet<(Uuid, Uuid)>,
    pub post_reports: HashSet<(Uuid, Uuid)>,
    pub tags: HashSet<String>,
    pub eligibility_tokens: HashMap<String, Uuid>,
    pub eligibility_token_status: HashMap<String, String>,
    pub uploaded_files: HashMap<String, Vec<String>>,
    pub advertisements: Vec<AdvertisementRecord>,

    // --- Phase 4: Comments ---
    pub comments: Vec<CommentRecord>,
    pub comment_likes: HashSet<(Uuid, Uuid)>,
    pub comment_reports: HashSet<(Uuid, Uuid)>,

    // --- Phase 4: Chat ---
    pub chats: Vec<ChatRecord>,
    pub chat_messages: Vec<ChatMessageRecord>,

    // --- Phase 5: Economy ---
    pub wallets: HashMap<Uuid, Decimal>,
    pub transactions: Vec<TransactionRecord>,
    pub daily_actions_used: HashMap<(Uuid, String, String), u32>,
    pub gems: Vec<GemRecord>,
    pub minted_dates: HashSet<String>,
    pub shop_orders: Vec<ShopOrderRecord>,

    // --- Phase 6: Admin & Moderation ---
    pub moderation_tickets: Vec<ModerationTicketRecord>,
    pub content_visibility: HashMap<Uuid, String>,
    pub alpha_minted: bool,
}

impl MockState {
    /// Reset state to defaults (for test isolation)
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Look up a user by email address.
    pub fn find_user_by_email(&self, email: &str) -> Option<&User> {
        self.users.values().find(|u| u.email == email)
    }

    /// Invalidate all tokens belonging to a user.
    pub fn invalidate_user_tokens(&mut self, uid: &Uuid) {
        self.access_tokens.retain(|_, v| v != uid);
        self.refresh_tokens.retain(|_, v| v != uid);
    }

    /// Check if `follower` follows `followed`.
    pub fn is_following(&self, follower: &Uuid, followed: &Uuid) -> bool {
        self.follows.contains(&(*follower, *followed))
    }

    /// Check if `blocker` has blocked `blocked`.
    pub fn is_blocked_by(&self, blocker: &Uuid, blocked: &Uuid) -> bool {
        self.blocks.contains(&(*blocker, *blocked))
    }

    /// Check if two users are mutual follows (friends/peers).
    pub fn are_friends(&self, a: &Uuid, b: &Uuid) -> bool {
        self.is_following(a, b) && self.is_following(b, a)
    }

    /// Count followers of a user.
    pub fn count_followers(&self, uid: &Uuid) -> i32 {
        self.follows
            .iter()
            .filter(|(_, followed)| followed == uid)
            .count() as i32
    }

    /// Count users that a user follows.
    pub fn count_following(&self, uid: &Uuid) -> i32 {
        self.follows
            .iter()
            .filter(|(follower, _)| follower == uid)
            .count() as i32
    }

    /// Count mutual follows (friends) of a user.
    pub fn count_friends(&self, uid: &Uuid) -> i32 {
        self.follows
            .iter()
            .filter(|(follower, followed)| {
                follower == uid && self.follows.contains(&(*followed, *follower))
            })
            .count() as i32
    }

    /// Count users blocked by a user.
    pub fn count_blocked(&self, uid: &Uuid) -> i32 {
        self.blocks
            .iter()
            .filter(|(blocker, _)| blocker == uid)
            .count() as i32
    }

    /// Count reports against a user.
    pub fn count_reports(&self, uid: &Uuid) -> i32 {
        self.reports.iter().filter(|r| r.reported == *uid).count() as i32
    }

    /// Check if `reporter` has already reported `reported`.
    pub fn user_has_reported(&self, reporter: &Uuid, reported: &Uuid) -> bool {
        self.reports
            .iter()
            .any(|r| r.reporter == *reporter && r.reported == *reported)
    }

    /// Check if a user has any active reports against them.
    pub fn has_active_reports(&self, uid: &Uuid) -> bool {
        self.reports.iter().any(|r| r.reported == *uid)
    }

    /// Find user by username (case-insensitive exact match).
    pub fn find_user_by_username(&self, username: &str) -> Option<&User> {
        let lower = username.to_lowercase();
        self.users
            .values()
            .find(|u| u.username.to_lowercase() == lower)
    }

    /// Search users by partial username match (case-insensitive).
    pub fn search_users_by_username(&self, query: &str) -> Vec<&User> {
        let lower = query.to_lowercase();
        self.users
            .values()
            .filter(|u| u.username.to_lowercase().contains(&lower))
            .collect()
    }

    // ========================================================================
    // Phase 3: Post helpers
    // ========================================================================

    /// Convert a PostRecord to the GraphQL Post type.
    pub fn post_record_to_graphql(
        &self,
        record: &PostRecord,
        viewer_id: Option<Uuid>,
    ) -> crate::types::post::Post {
        use crate::types::post::{Post, PostUser};

        let author = self.users.get(&record.author_id);

        let amountlikes = self
            .post_likes
            .iter()
            .filter(|(_, pid)| *pid == record.id)
            .count() as i32;
        let amountdislikes = self
            .post_dislikes
            .iter()
            .filter(|(_, pid)| *pid == record.id)
            .count() as i32;
        let amountviews = self
            .post_views
            .iter()
            .filter(|(_, pid)| *pid == record.id)
            .count() as i32;
        let amountreports = self
            .post_reports
            .iter()
            .filter(|(_, pid)| *pid == record.id)
            .count() as i32;
        let amountcomments = self
            .comments
            .iter()
            .filter(|c| c.post_id == record.id && c.visibility_status == "VISIBLE")
            .count() as i32;
        let amounttrending = amountlikes * 2 + amountviews + amountcomments;

        let (isliked, isviewed, isdisliked, issaved, isreported) = match viewer_id {
            Some(uid) => (
                self.post_likes.contains(&(uid, record.id)),
                self.post_views.contains(&(uid, record.id)),
                self.post_dislikes.contains(&(uid, record.id)),
                self.post_saves.contains(&(uid, record.id)),
                self.post_reports.contains(&(uid, record.id)),
            ),
            None => (false, false, false, false, false),
        };

        let user = match author {
            Some(u) => {
                let (isfollowed, isfollowing, isfriend) = match viewer_id {
                    Some(vid) => {
                        let followed = self.follows.contains(&(vid, u.uid));
                        let following = self.follows.contains(&(u.uid, vid));
                        (followed, following, followed && following)
                    }
                    None => (false, false, false),
                };
                PostUser {
                    id: u.uid.to_string().into(),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    img: u.img.clone(),
                    isfollowed,
                    isfollowing,
                    isfriend,
                }
            }
            None => PostUser {
                id: record.author_id.to_string().into(),
                username: "unknown".into(),
                slug: "unknown".into(),
                img: None,
                isfollowed: false,
                isfollowing: false,
                isfriend: false,
            },
        };

        let has_reports = amountreports > 0;

        Post {
            id: record.id.to_string().into(),
            contenttype: record.contenttype.clone(),
            title: record.title.clone(),
            media: if record.media.is_empty() {
                None
            } else {
                Some(record.media.clone())
            },
            cover: if record.cover.is_empty() {
                None
            } else {
                Some(record.cover.clone())
            },
            mediadescription: if record.mediadescription.is_empty() {
                None
            } else {
                Some(record.mediadescription.clone())
            },
            createdat: record.created_at.clone(),
            visibility_status: {
                let mod_vis = self.get_visibility(&record.id);
                if mod_vis != "NORMAL" {
                    Some(mod_vis.to_string())
                } else {
                    Some(record.visibility_status.clone())
                }
            },
            is_hidden_for_users: Some(self.get_visibility(&record.id) == "HIDDEN"),
            has_active_reports: Some(has_reports),
            amountreports,
            amountlikes,
            amountviews,
            amountcomments,
            amountdislikes,
            amounttrending: Some(amounttrending),
            isliked,
            isviewed,
            isreported,
            isdisliked,
            issaved,
            tags: record.tags.clone(),
            url: format!("/post/{}", record.id),
            user,
        }
    }

    /// Apply filters to posts and return matching records.
    #[allow(clippy::too_many_arguments)]
    pub fn filter_posts(
        &self,
        viewer_id: Option<Uuid>,
        filter_by: &[crate::types::post::PostFilterType],
        _content_filter_by: Option<crate::types::post::ContentFilterType>,
        ignore_list: Option<crate::types::post::IgnoreOption>,
        userid: Option<Uuid>,
        postid: Option<Uuid>,
        title: Option<&str>,
        tag: Option<&str>,
    ) -> Vec<&PostRecord> {
        use crate::types::post::{IgnoreOption, PostFilterType};

        self.posts
            .iter()
            .filter(|p| {
                if p.visibility_status != "VISIBLE" {
                    return false;
                }

                // Content visibility from moderation
                let mod_vis = self.get_visibility(&p.id);
                if mod_vis == "ILLEGAL" {
                    return false;
                }

                // Exclude advertisement posts from normal feed
                if self.advertisements.iter().any(|ad| ad.post_id == p.id) {
                    return false;
                }

                // Block list filtering
                if ignore_list != Some(IgnoreOption::No)
                    && let Some(vid) = viewer_id
                    && self.blocks.contains(&(vid, p.author_id))
                {
                    return false;
                }

                // Exclude posts by deleted users
                if self.deleted_users.contains(&p.author_id) {
                    return false;
                }

                // Single post by ID
                if let Some(pid) = postid {
                    return p.id == pid;
                }

                // User-scoped
                if let Some(uid) = userid
                    && p.author_id != uid
                {
                    return false;
                }

                // Content type / social filters
                for f in filter_by {
                    match f {
                        PostFilterType::Image => {
                            if p.contenttype != "image" {
                                return false;
                            }
                        }
                        PostFilterType::Audio => {
                            if p.contenttype != "audio" {
                                return false;
                            }
                        }
                        PostFilterType::Video => {
                            if p.contenttype != "video" {
                                return false;
                            }
                        }
                        PostFilterType::Text => {
                            if p.contenttype != "text" {
                                return false;
                            }
                        }
                        PostFilterType::Followed => {
                            if let Some(vid) = viewer_id {
                                if !self.follows.contains(&(vid, p.author_id)) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        PostFilterType::Follower => {
                            if let Some(vid) = viewer_id {
                                if !self.follows.contains(&(p.author_id, vid)) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        PostFilterType::Friends => {
                            if let Some(vid) = viewer_id {
                                let mutual = self.follows.contains(&(vid, p.author_id))
                                    && self.follows.contains(&(p.author_id, vid));
                                if !mutual {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        PostFilterType::Viewed => {
                            if let Some(vid) = viewer_id {
                                if !self.post_views.contains(&(vid, p.id)) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                    }
                }

                // Title search
                if let Some(t) = title
                    && !t.is_empty()
                    && !p.title.to_lowercase().contains(&t.to_lowercase())
                {
                    return false;
                }

                // Tag filter
                if let Some(tg) = tag
                    && !tg.is_empty()
                {
                    let tg_lower = tg.to_lowercase();
                    if !p.tags.iter().any(|pt| pt.to_lowercase() == tg_lower) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    // ========================================================================
    // Phase 4: Comment helpers
    // ========================================================================

    /// Convert a CommentRecord to the GraphQL Comment type.
    pub fn comment_record_to_graphql(
        &self,
        record: &CommentRecord,
        viewer_id: Option<Uuid>,
    ) -> crate::types::comment::Comment {
        use crate::types::comment::{Comment, CommentUser};

        let author = self.users.get(&record.author_id);

        let amountlikes = self
            .comment_likes
            .iter()
            .filter(|(_, cid)| *cid == record.id)
            .count() as i32;

        let amountreplies = self
            .comments
            .iter()
            .filter(|c| c.parent_id == Some(record.id) && c.visibility_status == "VISIBLE")
            .count() as i32;

        let isliked = match viewer_id {
            Some(uid) => self.comment_likes.contains(&(uid, record.id)),
            None => false,
        };

        let user = match author {
            Some(u) => {
                let (isfollowed, isfollowing) = match viewer_id {
                    Some(vid) => (
                        self.follows.contains(&(vid, u.uid)),
                        self.follows.contains(&(u.uid, vid)),
                    ),
                    None => (false, false),
                };
                CommentUser {
                    id: u.uid.to_string().into(),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    img: u.img.clone(),
                    isfollowed,
                    isfollowing,
                }
            }
            None => CommentUser {
                id: record.author_id.to_string().into(),
                username: "unknown".into(),
                slug: "unknown".into(),
                img: None,
                isfollowed: false,
                isfollowing: false,
            },
        };

        let mod_vis = self.get_visibility(&record.id);

        Comment {
            commentid: record.id.to_string().into(),
            userid: record.author_id.to_string().into(),
            postid: record.post_id.to_string().into(),
            parentid: record.parent_id.map(|p| p.to_string().into()),
            content: record.content.clone(),
            createdat: record.created_at.clone(),
            amountlikes,
            amountreplies,
            isliked,
            user,
            visibility_status: Some(mod_vis.to_string()),
            is_hidden_for_users: Some(mod_vis == "HIDDEN"),
        }
    }

    // ========================================================================
    // Phase 4: Chat helpers
    // ========================================================================

    /// Convert a ChatRecord to the GraphQL Chat type.
    pub fn chat_record_to_graphql(&self, record: &ChatRecord) -> crate::types::chat::Chat {
        use crate::types::chat::{Chat, ChatMessage, ChatParticipant};

        let chatparticipants: Vec<ChatParticipant> = record
            .participant_ids
            .iter()
            .filter_map(|uid| {
                self.users.get(uid).map(|u| ChatParticipant {
                    userid: u.uid.to_string(),
                    img: u.img.clone(),
                    username: u.username.clone(),
                    slug: Some(u.slug.clone()),
                    hasaccess: Some(true),
                })
            })
            .collect();

        let chatmessages: Vec<ChatMessage> = self
            .chat_messages
            .iter()
            .filter(|m| m.chat_id == record.id)
            .map(|m| ChatMessage {
                id: m.id.to_string(),
                senderid: m.sender_id.to_string(),
                chatid: m.chat_id.to_string(),
                content: m.content.clone(),
                createdat: m.created_at.clone(),
            })
            .collect();

        Chat {
            id: record.id.to_string(),
            name: record.name.clone(),
            image: record.image.clone(),
            createdat: record.created_at.clone(),
            updatedat: record.updated_at.clone(),
            chatmessages,
            chatparticipants,
        }
    }

    // ========================================================================
    // Phase 5: Economy helpers
    // ========================================================================

    /// Convert a TransactionRecord to the GraphQL TransactionHistoryItem type.
    pub fn transaction_record_to_graphql(
        &self,
        record: &TransactionRecord,
    ) -> crate::types::wallet::TransactionHistoryItem {
        use crate::types::wallet::{TransactionFees, TransactionHistoryItem, TransactionUser};

        let resolve_user = |uid: &Uuid| -> TransactionUser {
            match self.users.get(uid) {
                Some(u) => TransactionUser {
                    userid: u.uid.to_string(),
                    img: u.img.clone(),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    visibility_status: Some("VISIBLE".into()),
                    has_active_reports: Some(false),
                    is_hidden_for_users: Some(false),
                },
                None => TransactionUser {
                    userid: uid.to_string(),
                    img: None,
                    username: "System".into(),
                    slug: "system".into(),
                    visibility_status: Some("VISIBLE".into()),
                    has_active_reports: Some(false),
                    is_hidden_for_users: Some(false),
                },
            }
        };

        TransactionHistoryItem {
            transaction_id: record.id.to_string(),
            operationid: record.operation_id.to_string(),
            transaction_category: record.category,
            transactiontype: record.transaction_type.clone(),
            tokenamount: record.token_amount.to_string(),
            net_token_amount: record.net_token_amount.to_string(),
            message: record.message.clone(),
            createdat: record.created_at.clone(),
            sender: resolve_user(&record.sender_id),
            recipient: resolve_user(&record.recipient_id),
            fees: record.fees.as_ref().map(|f| TransactionFees {
                total: f.total,
                burn: f.burn,
                peer: f.peer,
                inviter: f.inviter,
            }),
        }
    }

    /// Convert an AdvertisementRecord to the AdvertisementPost GraphQL type.
    pub fn ad_record_to_graphql(
        &self,
        record: &AdvertisementRecord,
        viewer_id: Option<Uuid>,
    ) -> Option<crate::types::ad::AdvertisementPost> {
        use crate::types::ad::{AdvCreator, AdvertisementPost};

        let post_record = self.posts.iter().find(|p| p.id == record.post_id)?;
        let post = self.post_record_to_graphql(post_record, viewer_id);

        Some(AdvertisementPost {
            post,
            advertisement: AdvCreator {
                advertisementid: record.id.to_string().into(),
                advertisementtype: match record.ad_type {
                    AdvertisementType::Basic => "BASIC".into(),
                    AdvertisementType::Pinned => "PINNED".into(),
                },
                startdate: record.start_date.clone(),
                enddate: record.end_date.clone(),
                createdat: Some(record.created_at.clone()),
                user: None,
            },
        })
    }

    /// Convert an AdvertisementRecord to the full Advertisement GraphQL type (for history).
    pub fn ad_record_to_full_graphql(
        &self,
        record: &AdvertisementRecord,
    ) -> Option<crate::types::ad::Advertisement> {
        use crate::types::ad::Advertisement;
        use crate::types::user::{ContentVisibilityStatus, ProfileUserGql};

        let post_record = self.posts.iter().find(|p| p.id == record.post_id)?;
        let post = self.post_record_to_graphql(post_record, Some(record.advertiser_id));

        let user = self
            .users
            .get(&record.advertiser_id)
            .map(|u| ProfileUserGql {
                userid: u.uid.to_string().into(),
                username: u.username.clone(),
                slug: u.slug_num,
                img: u.img.clone(),
                visibility_status: ContentVisibilityStatus::Normal,
                is_hidden_for_users: false,
                has_active_reports: self.has_active_reports(&u.uid),
                isfollowed: false,
                isfollowing: false,
            })?;

        let cost_f64 = record.token_cost.to_string().parse::<f64>().unwrap_or(0.0);

        Some(Advertisement {
            id: record.id.to_string().into(),
            created_at: record.created_at.clone(),
            ad_type: record.ad_type,
            timeframe_start: record.start_date.clone(),
            timeframe_end: record.end_date.clone(),
            total_token_cost: cost_f64,
            total_euro_cost: 0.0,
            gems_earned: 0.0,
            amount_likes: 0,
            amount_views: 0,
            amount_comments: 0,
            amount_dislikes: 0,
            amount_reports: 0,
            user,
            post,
        })
    }

    /// Check if a daily free action is available.
    pub fn is_daily_free(&self, user_id: Uuid, action: &str) -> bool {
        let today = today_date_string();
        let key = (user_id, today, action.to_string());
        let used = self.daily_actions_used.get(&key).copied().unwrap_or(0);
        let limit = match action {
            "post" => FREE_POSTS,
            "like" => FREE_LIKES,
            "comment" => FREE_COMMENTS,
            "dislike" => FREE_DISLIKES,
            _ => 0,
        };
        used < limit
    }

    /// Record use of a daily action.
    pub fn use_daily_action(&mut self, user_id: Uuid, action: &str) {
        let today = today_date_string();
        let key = (user_id, today, action.to_string());
        *self.daily_actions_used.entry(key).or_insert(0) += 1;
    }

    /// Try to perform a paid action: use free allowance if available, otherwise deduct tokens.
    pub fn try_deduct_for_action(
        &mut self,
        user_id: Uuid,
        action: &str,
    ) -> Result<bool, &'static str> {
        if self.is_daily_free(user_id, action) {
            self.use_daily_action(user_id, action);
            return Ok(true);
        }

        let price = match action {
            "post" => POST_PRICE,
            "like" => LIKE_PRICE,
            "dislike" => DISLIKE_PRICE,
            "comment" => COMMENT_PRICE,
            _ => return Ok(false),
        };

        let balance = self.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if balance < price {
            return Err("51301");
        }

        *self.wallets.entry(user_id).or_insert(Decimal::ZERO) -= price;
        *self
            .wallets
            .entry(SYSTEM_PEER_ACCOUNT)
            .or_insert(Decimal::ZERO) += price;
        self.use_daily_action(user_id, action);

        Ok(false)
    }

    // ========================================================================
    // Phase 6: Admin & Moderation helpers
    // ========================================================================

    /// Check if a UUID is a system account.
    pub fn is_system_account(&self, uid: Uuid) -> bool {
        uid == SYSTEM_BURN_ACCOUNT
            || uid == SYSTEM_PEER_ACCOUNT
            || uid == SYSTEM_SHOP_ACCOUNT
            || uid == SYSTEM_MINT_ACCOUNT
    }

    /// Get content visibility status string (defaults to "NORMAL").
    pub fn get_visibility(&self, content_id: &Uuid) -> &str {
        self.content_visibility
            .get(content_id)
            .map(|s| s.as_str())
            .unwrap_or("NORMAL")
    }

    /// Check if content is visible (not ILLEGAL).
    pub fn is_content_visible(&self, content_id: &Uuid) -> bool {
        self.get_visibility(content_id) != "ILLEGAL"
    }

    /// Resolve a user UUID into BasicUserInfo.
    pub fn find_user_basic_info(
        &self,
        uid: &Uuid,
    ) -> Option<crate::types::moderation::BasicUserInfo> {
        let u = self.users.get(uid)?;
        let vis = self.get_visibility(&u.uid);
        Some(crate::types::moderation::BasicUserInfo {
            userid: u.uid.to_string(),
            img: u.img.clone(),
            username: u.username.clone(),
            slug: u.slug.clone(),
            biography: u.biography.clone(),
            visibility_status: Some(vis.to_string()),
            has_active_reports: Some(self.has_active_reports(&u.uid)),
            is_hidden_for_users: Some(vis == "HIDDEN"),
            updatedat: Some(u.updated_at.clone()),
        })
    }

    /// Resolve a moderation ticket record into its GraphQL representation.
    pub fn resolve_moderation_item(
        &self,
        ticket: &ModerationTicketRecord,
    ) -> crate::types::moderation::ModerationItem {
        use crate::types::moderation::{ModerationItem, TargetContent};

        let target_content = match ticket.target_type.as_str() {
            "post" => {
                let post = self
                    .posts
                    .iter()
                    .find(|p| p.id == ticket.target_content_id)
                    .map(|p| self.post_record_to_graphql(p, None));
                TargetContent {
                    post,
                    comment: None,
                    user: None,
                }
            }
            "comment" => {
                let comment = self
                    .comments
                    .iter()
                    .find(|c| c.id == ticket.target_content_id)
                    .map(|c| self.comment_record_to_graphql(c, None));
                TargetContent {
                    post: None,
                    comment,
                    user: None,
                }
            }
            "user" => {
                let user = self.find_user_basic_info(&ticket.target_content_id);
                TargetContent {
                    post: None,
                    comment: None,
                    user,
                }
            }
            _ => TargetContent {
                post: None,
                comment: None,
                user: None,
            },
        };

        let reporters: Vec<_> = ticket
            .reporter_ids
            .iter()
            .filter_map(|uid| self.find_user_basic_info(uid))
            .collect();

        let moderated_by = ticket
            .moderated_by
            .and_then(|uid| self.find_user_basic_info(&uid));

        ModerationItem {
            moderation_ticket_id: ticket.id.to_string().into(),
            target_content_id: ticket.target_content_id.to_string().into(),
            targettype: ticket.target_type.clone(),
            reportscount: ticket.reports_count,
            status: ticket.status.clone(),
            createdat: ticket.created_at.clone(),
            targetcontent: target_content,
            reporters,
            moderated_by,
        }
    }

    /// Find or create a moderation ticket for reported content.
    pub fn report_content(
        &mut self,
        target_id: Uuid,
        target_type: &str,
        reporter_id: Uuid,
    ) -> Result<bool, &'static str> {
        // Check if restored content (cannot be re-reported)
        if self
            .moderation_tickets
            .iter()
            .any(|t| t.target_content_id == target_id && t.status == "restored")
        {
            return Err("32104");
        }

        // Check if reporter already reported this content
        if self
            .moderation_tickets
            .iter()
            .any(|t| t.target_content_id == target_id && t.reporter_ids.contains(&reporter_id))
        {
            return Err("32104");
        }

        // Find existing waiting ticket for this content
        if let Some(ticket) = self
            .moderation_tickets
            .iter_mut()
            .find(|t| t.target_content_id == target_id && t.status == "waiting_for_review")
        {
            ticket.reporter_ids.push(reporter_id);
            ticket.reports_count += 1;
            return Ok(false);
        }

        // Create new ticket
        let ticket = ModerationTicketRecord {
            id: Uuid::new_v4(),
            target_content_id: target_id,
            target_type: target_type.to_string(),
            reporter_ids: vec![reporter_id],
            reports_count: 1,
            status: "waiting_for_review".to_string(),
            moderated_by: None,
            created_at: today_date_string(),
        };
        self.moderation_tickets.push(ticket);
        Ok(true)
    }

    /// Search users with admin-specific filters.
    #[allow(clippy::too_many_arguments)]
    pub fn search_users_admin(
        &self,
        userid: Option<&async_graphql::ID>,
        email: Option<&str>,
        username: Option<&str>,
        status: Option<i32>,
        verified: Option<i32>,
        ip: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Vec<crate::types::admin::AdminUser> {
        let results: Vec<_> = self
            .users
            .values()
            .filter(|u| {
                if let Some(uid) = userid
                    && u.uid.to_string() != uid.as_str()
                {
                    return false;
                }
                if let Some(e) = email
                    && !u.email.to_lowercase().contains(&e.to_lowercase())
                {
                    return false;
                }
                if let Some(un) = username
                    && !u.username.to_lowercase().contains(&un.to_lowercase())
                {
                    return false;
                }
                if let Some(s) = status
                    && u.status as i32 != s
                {
                    return false;
                }
                if let Some(v) = verified {
                    let is_verified = i32::from(self.verified_users.contains(&u.uid));
                    if is_verified != v {
                        return false;
                    }
                }
                if let Some(ip_filter) = ip {
                    match &u.ip {
                        Some(user_ip) if user_ip == ip_filter => {}
                        _ => return false,
                    }
                }
                true
            })
            .skip(offset)
            .take(limit)
            .map(|u| {
                let vis = self.get_visibility(&u.uid);
                let liquidity = self.wallets.get(&u.uid).copied();
                crate::types::admin::AdminUser {
                    id: u.uid.to_string().into(),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    img: u.img.clone(),
                    biography: u.biography.clone(),
                    status: Some(u.status as i32),
                    visibility_status: Some(vis.to_string()),
                    has_active_reports: Some(self.has_active_reports(&u.uid)),
                    is_hidden_for_users: Some(vis == "HIDDEN"),
                    email: Some(u.email.clone()),
                    verified: Some(if self.verified_users.contains(&u.uid) {
                        1
                    } else {
                        0
                    }),
                    roles_mask: Some(u.role as i32),
                    ip: u.ip.clone(),
                    liquidity,
                    situation: None,
                }
            })
            .collect();
        results
    }

    /// Get username by UUID.
    pub fn get_username(&self, uid: Uuid) -> String {
        self.users
            .get(&uid)
            .map(|u| u.username.clone())
            .unwrap_or_else(|| "unknown".into())
    }

    /// Get admin post comments with subcomments and visibility info.
    pub fn get_admin_post_comments(
        &self,
        post_id_str: &str,
        offset: usize,
        limit: usize,
    ) -> Vec<crate::types::admin::PostCommentsData> {
        use crate::types::admin::{PostCommentsData, PostSubCommentsData};
        use crate::types::user::ContentVisibilityStatus;

        let post_id = match Uuid::parse_str(post_id_str) {
            Ok(id) => id,
            Err(_) => return vec![],
        };

        // Get top-level comments for this post
        let top_level: Vec<_> = self
            .comments
            .iter()
            .filter(|c| c.post_id == post_id && c.parent_id.is_none())
            .skip(offset)
            .take(limit)
            .collect();

        top_level
            .iter()
            .map(|c| {
                let vis_str = self.get_visibility(&c.id);
                let vis = match vis_str {
                    "HIDDEN" => ContentVisibilityStatus::Hidden,
                    "ILLEGAL" => ContentVisibilityStatus::Illegal,
                    _ => ContentVisibilityStatus::Normal,
                };

                let amountlikes = self
                    .comment_likes
                    .iter()
                    .filter(|(_, cid)| *cid == c.id)
                    .count();

                let subcomments: Vec<PostSubCommentsData> = self
                    .comments
                    .iter()
                    .filter(|sc| sc.parent_id == Some(c.id))
                    .map(|sc| {
                        let sc_vis_str = self.get_visibility(&sc.id);
                        let sc_vis = match sc_vis_str {
                            "HIDDEN" => ContentVisibilityStatus::Hidden,
                            "ILLEGAL" => ContentVisibilityStatus::Illegal,
                            _ => ContentVisibilityStatus::Normal,
                        };
                        let sc_likes = self
                            .comment_likes
                            .iter()
                            .filter(|(_, cid)| *cid == sc.id)
                            .count();
                        let sc_replies = self
                            .comments
                            .iter()
                            .filter(|r| r.parent_id == Some(sc.id))
                            .count();
                        PostSubCommentsData {
                            commentid: Some(sc.id.to_string().into()),
                            userid: Some(sc.author_id.to_string().into()),
                            postid: Some(sc.post_id.to_string().into()),
                            parentid: sc.parent_id.map(|p| p.to_string().into()),
                            content: Some(sc.content.clone()),
                            createdat: Some(sc.created_at.clone()),
                            amountlikes: Some(Decimal::from(sc_likes as i64)),
                            amountreplies: Some(Decimal::from(sc_replies as i64)),
                            isliked: Some(false),
                            user: self.find_user_basic_info(&sc.author_id),
                            visibility_status: sc_vis,
                            is_hidden_for_users: sc_vis_str == "HIDDEN",
                        }
                    })
                    .collect();

                PostCommentsData {
                    commentid: Some(c.id.to_string().into()),
                    userid: Some(c.author_id.to_string().into()),
                    postid: Some(c.post_id.to_string().into()),
                    parentid: None,
                    content: Some(c.content.clone()),
                    createdat: Some(c.created_at.clone()),
                    amountlikes: Some(Decimal::from(amountlikes as i64)),
                    isliked: Some(false),
                    user: self.find_user_basic_info(&c.author_id),
                    subcomments: if subcomments.is_empty() {
                        None
                    } else {
                        Some(subcomments)
                    },
                    visibility_status: vis,
                    is_hidden_for_users: vis_str == "HIDDEN",
                }
            })
            .collect()
    }

    /// Aggregate gem records into DailyGemStatusData (d0–d7, w0, m0, y0).
    pub fn aggregate_gems_by_period(&self) -> crate::types::admin_gems::DailyGemStatusData {
        let today = chrono::Utc::now().date_naive();

        let gem_date = |g: &GemRecord| -> Option<chrono::NaiveDate> {
            chrono::NaiveDate::parse_from_str(&g.created_at[..10], "%Y-%m-%d").ok()
        };

        let sum_for = |filter: &dyn Fn(chrono::NaiveDate) -> bool| -> Decimal {
            let total: f64 = self
                .gems
                .iter()
                .filter_map(|g| gem_date(g).map(|d| (d, g.gems)))
                .filter(|(d, _)| filter(*d))
                .map(|(_, gems)| gems)
                .sum();
            Decimal::from_f64_retain(total).unwrap_or(Decimal::ZERO)
        };

        let day = |n: i64| -> Decimal {
            let target = today - chrono::Duration::days(n);
            sum_for(&|d| d == target)
        };

        let week_start =
            today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
        let month_start = today.with_day(1).unwrap_or(today);
        let year_start = chrono::NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap_or(today);

        crate::types::admin_gems::DailyGemStatusData {
            d0: day(0),
            d1: day(1),
            d2: day(2),
            d3: day(3),
            d4: day(4),
            d5: day(5),
            d6: day(6),
            d7: day(7),
            w0: sum_for(&|d| d >= week_start && d <= today),
            m0: sum_for(&|d| d >= month_start && d <= today),
            y0: sum_for(&|d| d >= year_start && d <= today),
        }
    }

    /// Get per-user gem totals for a specific day filter.
    pub fn get_gems_for_day(
        &self,
        day: &crate::types::wallet::DayFilterType,
    ) -> (
        Vec<crate::types::admin_gems::DailyGemsResultsUserData>,
        Decimal,
    ) {
        let target_date = day_filter_to_date(day);
        let mut user_gems: std::collections::HashMap<Uuid, f64> = std::collections::HashMap::new();

        for g in &self.gems {
            if g.created_at.starts_with(&target_date) {
                *user_gems.entry(g.user_id).or_default() += g.gems;
            }
        }

        let total: f64 = user_gems.values().sum();
        let results: Vec<_> = user_gems
            .into_iter()
            .map(
                |(uid, gems)| crate::types::admin_gems::DailyGemsResultsUserData {
                    userid: Some(uid.to_string().into()),
                    pkey: Some(uid.to_string().into()),
                    gems: Some(Decimal::from_f64_retain(gems).unwrap_or(Decimal::ZERO)),
                },
            )
            .collect();

        (
            results,
            Decimal::from_f64_retain(total).unwrap_or(Decimal::ZERO),
        )
    }

    /// Convert pending interactions to gem records. Returns count converted.
    pub fn convert_interactions_to_gems(&mut self) -> usize {
        let now = today_date_string();
        let mut count = 0usize;

        // Scan post interactions and create gem records for unprocessed ones
        // We track which (user, post, action) combos already have gem records
        let existing_gems: std::collections::HashSet<(Uuid, Uuid, String)> = self
            .gems
            .iter()
            .map(|g| (g.user_id, g.post_id, g.action.clone()))
            .collect();

        // Process likes
        let likes: Vec<(Uuid, Uuid)> = self.post_likes.iter().copied().collect();
        for (liker_id, post_id) in likes {
            if let Some(post) = self.posts.iter().find(|p| p.id == post_id) {
                let author_id = post.author_id;
                if author_id != liker_id
                    && !existing_gems.contains(&(author_id, post_id, "like".to_string()))
                {
                    self.gems.push(GemRecord {
                        user_id: author_id,
                        post_id,
                        from_user_id: liker_id,
                        gems: LIKE_GEM_RETURN,
                        action: "like".into(),
                        created_at: format!("{now}T00:00:00Z"),
                    });
                    count += 1;
                }
            }
        }

        // Process views
        let views: Vec<(Uuid, Uuid)> = self.post_views.iter().copied().collect();
        for (viewer_id, post_id) in views {
            if let Some(post) = self.posts.iter().find(|p| p.id == post_id) {
                let author_id = post.author_id;
                if author_id != viewer_id
                    && !existing_gems.contains(&(author_id, post_id, "view".to_string()))
                {
                    self.gems.push(GemRecord {
                        user_id: author_id,
                        post_id,
                        from_user_id: viewer_id,
                        gems: VIEW_GEM_RETURN,
                        action: "view".into(),
                        created_at: format!("{now}T00:00:00Z"),
                    });
                    count += 1;
                }
            }
        }

        // Process dislikes (negative gems for the post author)
        let dislikes: Vec<(Uuid, Uuid)> = self.post_dislikes.iter().copied().collect();
        for (disliker_id, post_id) in dislikes {
            if let Some(post) = self.posts.iter().find(|p| p.id == post_id) {
                let author_id = post.author_id;
                if author_id != disliker_id
                    && !existing_gems.contains(&(author_id, post_id, "dislike".to_string()))
                {
                    self.gems.push(GemRecord {
                        user_id: author_id,
                        post_id,
                        from_user_id: disliker_id,
                        gems: DISLIKE_GEM_RETURN,
                        action: "dislike".into(),
                        created_at: format!("{now}T00:00:00Z"),
                    });
                    count += 1;
                }
            }
        }

        // Process comments (gems for the post author per unique commenter)
        let comment_pairs: Vec<(Uuid, Uuid)> = self
            .comments
            .iter()
            .map(|c| (c.author_id, c.post_id))
            .collect();
        for (commenter_id, post_id) in comment_pairs {
            if let Some(post) = self.posts.iter().find(|p| p.id == post_id) {
                let author_id = post.author_id;
                if author_id != commenter_id
                    && !existing_gems.contains(&(author_id, post_id, "comment".to_string()))
                {
                    self.gems.push(GemRecord {
                        user_id: author_id,
                        post_id,
                        from_user_id: commenter_id,
                        gems: COMMENT_GEM_RETURN,
                        action: "comment".into(),
                        created_at: format!("{now}T00:00:00Z"),
                    });
                    count += 1;
                }
            }
        }

        count
    }

    /// Distribute tokens from gems for a specific date.
    pub fn distribute_gems_to_tokens(
        &mut self,
        date: &str,
        daily_token_amount: f64,
    ) -> Option<(crate::types::admin_gems::GemstersData, i32)> {
        use crate::types::admin_gems::*;

        // Gather user gems for the date
        let mut user_gems: std::collections::HashMap<Uuid, f64> = std::collections::HashMap::new();
        for g in &self.gems {
            if g.created_at.starts_with(date) {
                *user_gems.entry(g.user_id).or_default() += g.gems;
            }
        }

        if user_gems.is_empty() {
            return None;
        }

        let total_gems: f64 = user_gems.values().sum();
        if total_gems <= 0.0 {
            return None;
        }

        let gemsintoken = daily_token_amount / total_gems;

        let mut user_statuses = Vec::new();
        for (&uid, &gems) in &user_gems {
            let tokens = gems * gemsintoken;
            let percentage = if total_gems > 0.0 {
                (gems / total_gems) * 100.0
            } else {
                0.0
            };

            let token_dec = Decimal::from_f64_retain(tokens).unwrap_or(Decimal::ZERO);
            *self.wallets.entry(uid).or_insert(Decimal::ZERO) += token_dec;
            *self
                .wallets
                .entry(SYSTEM_MINT_ACCOUNT)
                .or_insert(Decimal::ZERO) -= token_dec;

            // Get gem details for this user on this date
            let details: Vec<GemstersUserStatusDetails> = self
                .gems
                .iter()
                .filter(|g| g.user_id == uid && g.created_at.starts_with(date))
                .map(|g| GemstersUserStatusDetails {
                    gemid: Some(Uuid::new_v4().to_string().into()),
                    userid: Some(g.user_id.to_string().into()),
                    postid: Some(g.post_id.to_string().into()),
                    fromid: Some(g.from_user_id.to_string().into()),
                    gems: Some(Decimal::from_f64_retain(g.gems).unwrap_or(Decimal::ZERO)),
                    numbers: Some(Decimal::from(1)),
                    whereby: Some(Decimal::from_f64_retain(g.gems).unwrap_or(Decimal::ZERO)),
                    createdat: Some(g.created_at.clone()),
                })
                .collect();

            user_statuses.push(GemstersUserStatus {
                userid: Some(uid.to_string().into()),
                gems: Some(Decimal::from_f64_retain(gems).unwrap_or(Decimal::ZERO)),
                tokens: Some(token_dec),
                percentage: Some(Decimal::from_f64_retain(percentage).unwrap_or(Decimal::ZERO)),
                details: if details.is_empty() {
                    None
                } else {
                    Some(details)
                },
            });
        }

        let counter = user_statuses.len() as i32;

        let data = GemstersData {
            win_status: Some(WinStatus {
                total_gems: Decimal::from_f64_retain(total_gems).unwrap_or(Decimal::ZERO),
                gemsintoken: Decimal::from_f64_retain(gemsintoken).unwrap_or(Decimal::ZERO),
                bestatigung: Decimal::from(1),
            }),
            user_status: Some(user_statuses),
        };

        Some((data, counter))
    }
}

/// Get today's date as YYYY-MM-DD string.
pub fn today_date_string() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// Check if an advertisement is currently active.
pub fn is_ad_active(ad: &AdvertisementRecord, today: &str) -> bool {
    ad.start_date.as_str() <= today && today <= ad.end_date.as_str()
}

/// Convert DayFilterType to a YYYY-MM-DD date string.
pub fn day_filter_to_date(day: &crate::types::wallet::DayFilterType) -> String {
    use crate::types::wallet::DayFilterType;
    let today = chrono::Utc::now().date_naive();
    let target = match day {
        DayFilterType::D0 => today,
        DayFilterType::D1 => today - chrono::Duration::days(1),
        DayFilterType::D2 => today - chrono::Duration::days(2),
        DayFilterType::D3 => today - chrono::Duration::days(3),
        DayFilterType::D4 => today - chrono::Duration::days(4),
        DayFilterType::D5 => today - chrono::Duration::days(5),
        DayFilterType::D6 => today - chrono::Duration::days(6),
        DayFilterType::D7 => today - chrono::Duration::days(7),
        DayFilterType::W0 | DayFilterType::M0 | DayFilterType::Y0 => today,
    };
    target.format("%Y-%m-%d").to_string()
}
