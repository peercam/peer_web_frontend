use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

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

/// Internal advertisement record.
#[derive(Debug, Clone)]
pub struct AdvertisementRecord {
    pub id: String,
    pub post_id: Uuid,
    pub advertisement_type: String,
    pub start_date: String,
    pub end_date: String,
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
    pub daily_comment_count: HashMap<(Uuid, String), u32>,

    // --- Phase 4: Chat ---
    pub chats: Vec<ChatRecord>,
    pub chat_messages: Vec<ChatMessageRecord>,
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
            visibility_status: Some(record.visibility_status.clone()),
            is_hidden_for_users: Some(record.visibility_status == "HIDDEN"),
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
}
