//! Generic GraphQL client for communicating with the Peer API.
//!
//! This module provides type-safe query and mutation helpers that:
//! - Serialize variables to JSON
//! - Send POST requests to the configured GraphQL endpoint
//! - Deserialize typed responses
//! - Handle the `{ data, errors }` envelope
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::api::graphql::mutate;
//! use crate::models::user::{ReferralVerifyResponse};
//!
//! let response: ReferralVerifyResponse = mutate(
//!     VERIFY_REFERRAL_MUTATION,
//!     serde_json::json!({ "referralString": "85d5f836-..." }),
//!     None, // No auth token for guest mutations
//! ).await?;
//! ```

use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use serde::de::DeserializeOwned;

use crate::models::common::{ApiError, GraphQLError};

/// GraphQL request envelope.
///
/// Matches the standard GraphQL-over-HTTP POST body format:
/// ```json
/// {
///   "query": "mutation { ... }",
///   "variables": { ... },
///   "operationName": "OptionalName"
/// }
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRequest<V: Serialize> {
    pub query: &'static str,
    pub variables: V,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<&'static str>,
}

impl<V: Serialize> GraphQLRequest<V> {
    /// Create a new GraphQL request.
    pub fn new(query: &'static str, variables: V) -> Self {
        Self {
            query,
            variables,
            operation_name: None,
        }
    }

    /// Create a request with an explicit operation name.
    pub fn with_operation(query: &'static str, variables: V, operation_name: &'static str) -> Self {
        Self {
            query,
            variables,
            operation_name: Some(operation_name),
        }
    }
}

/// GraphQL response envelope.
///
/// All GraphQL responses follow this shape:
/// ```json
/// {
///   "data": { ... },      // Present on success (may be null)
///   "errors": [ ... ]     // Present on error (may be absent)
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphQLError>,
}

impl<T> GraphQLResponse<T> {
    /// Check if the response contains any GraphQL errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the first error message, if any.
    pub fn first_error_message(&self) -> Option<&str> {
        self.errors.first().map(|e| e.message.as_str())
    }

    /// Extract data, returning an error if data is missing or errors are present.
    pub fn into_result(self) -> Result<T, ApiError> {
        if let Some(err) = self.errors.first() {
            return Err(ApiError::GraphQL(err.message.clone()));
        }

        self.data.ok_or_else(|| {
            ApiError::Unexpected("GraphQL response contained neither data nor errors".to_string())
        })
    }
}

// ============================================================================
// Server-side implementation (SSR only)
// ============================================================================

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;
    use std::env;
    use std::time::Duration;

    /// Default timeout for GraphQL requests (10 seconds).
    const DEFAULT_TIMEOUT_SECS: u64 = 10;

    /// Get the GraphQL endpoint URL from environment.
    ///
    /// Reads `GRAPHQL_ENDPOINT` env var, falling back to local mock backend.
    pub fn get_endpoint() -> String {
        env::var("GRAPHQL_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string())
    }

    /// Build a configured reqwest client.
    fn build_client() -> Result<reqwest::Client, ApiError> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| ApiError::Network(format!("Failed to build HTTP client: {}", e)))
    }

    /// Execute a GraphQL mutation and deserialize the response.
    ///
    /// # Type Parameters
    ///
    /// - `V`: The variables type (must implement `Serialize`)
    /// - `R`: The expected response data type (must implement `DeserializeOwned`)
    ///
    /// # Arguments
    ///
    /// - `query`: The GraphQL mutation string
    /// - `variables`: Mutation variables (will be serialized to JSON)
    /// - `auth_token`: Optional Bearer token for authenticated mutations
    pub async fn mutate<V, R>(
        query: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        execute_request(query, variables, auth_token).await
    }

    /// Execute a GraphQL query and deserialize the response.
    ///
    /// Identical to `mutate` but semantically for queries.
    pub async fn query<V, R>(
        query_str: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        execute_request(query_str, variables, auth_token).await
    }

    /// Internal: Execute a GraphQL request (query or mutation).
    async fn execute_request<V, R>(
        query: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        let client = build_client()?;
        let endpoint = get_endpoint();

        let request_body = GraphQLRequest::new(query, variables);

        let mut request = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Add authorization header if token provided
        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request
        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Network("Request timed out".to_string())
                } else if e.is_connect() {
                    ApiError::Network(format!("Failed to connect to {}: {}", endpoint, e))
                } else {
                    ApiError::Network(format!("Request failed: {}", e))
                }
            })?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Network(format!(
                "HTTP {} from {}: {}",
                status, endpoint, body
            )));
        }

        // Parse response body
        let response_text = response.text().await.map_err(|e| {
            ApiError::Deserialization(format!("Failed to read response body: {}", e))
        })?;

        // Deserialize GraphQL envelope
        let graphql_response: GraphQLResponse<R> =
            serde_json::from_str(&response_text).map_err(|e| {
                ApiError::Deserialization(format!(
                    "Failed to parse GraphQL response: {}. Body: {}",
                    e,
                    truncate_for_error(&response_text, 200)
                ))
            })?;

        // Extract data or convert errors
        graphql_response.into_result()
    }

    /// Truncate a string for error messages.
    fn truncate_for_error(s: &str, max_len: usize) -> &str {
        if s.len() <= max_len {
            s
        } else {
            &s[..max_len]
        }
    }
}

// Re-export SSR functions at module level
#[cfg(feature = "ssr")]
pub use ssr::*;

// ============================================================================
// GraphQL query/mutation string constants
// ============================================================================

/// Mutation: Verify a referral code.
pub const VERIFY_REFERRAL_MUTATION: &str = r#"
mutation VerifyReferralString($referralString: String!) {
    verifyReferralString(referralString: $referralString) {
        status
        ResponseCode
        affectedRows {
            uid
            username
            slug
            img
        }
    }
}
"#;

/// Mutation: Register a new user.
pub const REGISTER_MUTATION: &str = r#"
mutation Register($input: RegistrationInput!) {
    register(input: $input) {
        status
        ResponseCode
        userid
    }
}
"#;

/// Mutation: Verify a user account after registration.
pub const VERIFY_ACCOUNT_MUTATION: &str = r#"
mutation VerifyAccount($userId: ID!) {
    verifyAccount(userid: $userId) {
        status
        ResponseCode
    }
}
"#;

// ============================================================================
// Response wrapper types for GraphQL data field
// ============================================================================

/// Wrapper for the `verifyReferralString` mutation response.
///
/// The GraphQL response has shape:
/// ```json
/// { "data": { "verifyReferralString": { ... } } }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReferralData {
    pub verify_referral_string: crate::models::user::ReferralVerifyResponse,
}

/// Wrapper for the `register` mutation response.
#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub register: crate::models::user::RegisterResponse,
}

/// Wrapper for the `verifyAccount` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyAccountData {
    pub verify_account: crate::models::user::VerifyAccountResponse,
}

// ============================================================================
// Authentication mutations
// ============================================================================

/// Mutation: Login with email and password.
pub const LOGIN_MUTATION: &str = r#"
mutation Login($email: String!, $password: String!) {
    login(email: $email, password: $password) {
        status
        ResponseCode
        accessToken
        refreshToken
    }
}
"#;

/// Mutation: Refresh access token using refresh token.
pub const REFRESH_TOKEN_MUTATION: &str = r#"
mutation RefreshToken($refreshToken: String!) {
    refreshToken(refreshToken: $refreshToken) {
        status
        ResponseCode
        accessToken
        refreshToken
    }
}
"#;

/// Mutation: Logout and invalidate refresh token.
pub const LOGOUT_MUTATION: &str = r#"
mutation Logout($refreshToken: String!) {
    logout(refreshToken: $refreshToken) {
        status
        ResponseCode
    }
}
"#;

/// Wrapper for the `login` mutation response.
#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub login: crate::models::auth::AuthPayload,
}

/// Wrapper for the `refreshToken` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenData {
    pub refresh_token: crate::models::auth::AuthPayload,
}

/// Wrapper for the `logout` mutation response.
#[derive(Debug, Deserialize)]
pub struct LogoutData {
    pub logout: crate::models::auth::LogoutPayload,
}

// ============================================================================
// Posts queries and mutations
// ============================================================================

/// Query: List posts with filters.
pub const LIST_POSTS_QUERY: &str = r#"
query ListPosts(
    $filterBy: [PostFilterType!],
    $contentFilterBy: ContentFilterType,
    $sortBy: PostSortType,
    $title: String,
    $tag: String,
    $offset: Int,
    $limit: Int
) {
    listPosts(
        filterBy: $filterBy,
        contentFilterBy: $contentFilterBy,
        sortBy: $sortBy,
        title: $title,
        tag: $tag,
        offset: $offset,
        limit: $limit
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            id
            contenttype
            title
            media
            cover
            mediadescription
            createdat
            amountlikes
            amountviews
            amountcomments
            amountdislikes
            isliked
            isviewed
            isdisliked
            issaved
            tags
            user {
                id
                username
                slug
                img
                isfollowed
                isfollowing
                isfriend
            }
        }
    }
}
"#;

/// Query: List advertisement posts.
pub const LIST_AD_POSTS_QUERY: &str = r#"
query ListAdvertisementPosts(
    $offset: Int,
    $limit: Int,
    $contentFilterBy: ContentFilterType,
    $title: String,
    $tag: String
) {
    listAdvertisementPosts(
        offset: $offset,
        limit: $limit,
        contentFilterBy: $contentFilterBy,
        title: $title,
        tag: $tag
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            post {
                id
                contenttype
                title
                media
                cover
                mediadescription
                createdat
                amountlikes
                amountviews
                amountcomments
                amountdislikes
                isliked
                isviewed
                isdisliked
                issaved
                tags
                user {
                    id
                    username
                    slug
                    img
                    isfollowed
                    isfollowing
                    isfriend
                }
            }
            advertisement {
                advertisementid
                advertisementtype
                startdate
                enddate
            }
        }
    }
}
"#;

/// Mutation: Perform an action on a post (like, dislike, save, view).
pub const POST_ACTION_MUTATION: &str = r#"
mutation ResolvePostAction($postid: ID!, $action: PostActionType!) {
    resolvePostAction(postid: $postid, action: $action) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
    }
}
"#;

/// Query: Search users by username.
pub const SEARCH_USERS_QUERY: &str = r#"
query SearchUser($username: String!, $offset: Int, $limit: Int) {
    searchUser(username: $username, offset: $offset, limit: $limit) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            id
            username
            slug
            img
        }
    }
}
"#;

/// Query: Get user info.
pub const GET_USER_QUERY: &str = r#"
query GetUser($id: ID!) {
    getUser(id: $id) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            username
            slug
            img
            biography
            amountFollowers
            amountFollowing
            amountPeers
            userPreferences {
                contentFilteringSeverityLevel
            }
        }
    }
}
"#;

/// Wrapper for the `listPosts` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPostsData {
    pub list_posts: crate::models::post::PostListResponse,
}

/// Wrapper for the `listAdvertisementPosts` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAdPostsData {
    pub list_advertisement_posts: crate::models::post::AdListResponse,
}

/// Wrapper for the `resolvePostAction` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostActionData {
    pub resolve_post_action: crate::models::post::PostActionResponse,
}

/// Wrapper for the `searchUser` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchUserData {
    pub search_user: crate::models::post::UserSearchResponse,
}

/// Wrapper for the `getUser` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUserData {
    pub get_user: crate::models::user::UserInfoResponse,
}

// ============================================================================
// Guest Post & Comments queries and mutations
// ============================================================================

/// Query: Get a single post for guest viewing (no auth required).
pub const GUEST_POST_QUERY: &str = r#"
query GuestListPost($postid: ID!) {
    guestListPost(postid: $postid) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            contenttype
            title
            media
            cover
            mediadescription
            createdat
            amountlikes
            amountviews
            amountcomments
            amountdislikes
            tags
            user {
                id
                username
                slug
                img
            }
        }
    }
}
"#;

/// Query: Get a single post for authenticated viewing.
pub const GET_POST_QUERY: &str = r#"
query ListPost($postid: ID!) {
    listPosts(postid: $postid, limit: 1) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            contenttype
            title
            media
            cover
            mediadescription
            createdat
            amountlikes
            amountviews
            amountcomments
            amountdislikes
            amounttrending
            isliked
            isviewed
            isdisliked
            issaved
            isreported
            tags
            hasActiveReports
            visibilityStatus
            isHiddenForUsers
            user {
                id
                username
                slug
                img
                isfollowed
                isfollowing
                isfriend
            }
        }
    }
}
"#;

/// Query: List top-level comments for a post.
pub const LIST_COMMENTS_QUERY: &str = r#"
query ListComments($postid: ID!, $commentOffset: Int, $commentLimit: Int) {
    listComments(
        postid: $postid
        commentOffset: $commentOffset
        commentLimit: $commentLimit
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            commentid
            userid
            postid
            parentid
            content
            createdat
            amountlikes
            amountreplies
            isliked
            user {
                id
                username
                slug
                img
                isfollowed
                isfollowing
            }
        }
    }
}
"#;

/// Query: List child comments (replies) for a parent comment.
pub const LIST_CHILD_COMMENTS_QUERY: &str = r#"
query ListChildComments($parent: ID!, $offset: Int, $limit: Int) {
    listChildComments(parent: $parent, offset: $offset, limit: $limit) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            commentid
            userid
            postid
            parentid
            content
            createdat
            amountlikes
            amountreplies
            isliked
            user {
                id
                username
                slug
                img
                isfollowed
                isfollowing
            }
        }
    }
}
"#;

/// Mutation: Create a new comment or reply.
pub const CREATE_COMMENT_MUTATION: &str = r#"
mutation CreateComment($postid: ID!, $content: String!, $parentid: ID) {
    createComment(
        action: COMMENT
        postid: $postid
        content: $content
        parentid: $parentid
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            commentid
            userid
            content
            createdat
            amountlikes
            amountreplies
            isliked
            postid
            parentid
            user {
                id
                username
                slug
                img
                isfollowed
                isfollowing
            }
        }
    }
}
"#;

/// Mutation: Like a comment.
pub const LIKE_COMMENT_MUTATION: &str = r#"
mutation LikeComment($commentid: ID!) {
    likeComment(commentid: $commentid) {
        status
        ResponseCode
        ResponseMessage
    }
}
"#;

/// Mutation: Unlike a comment.
pub const UNLIKE_COMMENT_MUTATION: &str = r#"
mutation UnlikeComment($commentid: ID!) {
    unlikeComment(commentid: $commentid) {
        status
        ResponseCode
        ResponseMessage
    }
}
"#;

/// Wrapper for the `guestListPost` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestPostData {
    pub guest_list_post: crate::models::post::PostListResponse,
}

/// Wrapper for the `listPosts` single post query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPostData {
    pub list_posts: crate::models::post::PostListResponse,
}

/// Wrapper for the `listComments` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCommentsData {
    pub list_comments: crate::models::comment::CommentListResponse,
}

/// Wrapper for the `listChildComments` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListChildCommentsData {
    pub list_child_comments: crate::models::comment::CommentListResponse,
}

/// Wrapper for the `createComment` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentData {
    pub create_comment: crate::models::comment::CreateCommentResponse,
}

/// Wrapper for the `likeComment` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeCommentData {
    pub like_comment: crate::models::comment::LikeCommentResponse,
}

/// Wrapper for the `unlikeComment` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlikeCommentData {
    pub unlike_comment: crate::models::comment::LikeCommentResponse,
}

// ============================================================================
// Profile queries and mutations
// ============================================================================

/// Query: Get a user's profile.
pub const GET_PROFILE_QUERY: &str = r#"
query GetProfile($userid: ID, $contentFilterBy: ContentFilterType) {
    getProfile(userid: $userid, contentFilterBy: $contentFilterBy) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            username
            status
            slug
            img
            biography
            visibilityStatus
            isHiddenForUsers
            hasActiveReports
            iFollowThisUser
            thisUserFollowsMe
            isreported
            amountposts
            amounttrending
            amountfollowed
            amountfollower
            amountfriends
            amountblocked
            amountreports
        }
    }
}
"#;

/// Query: List followers and following for a user.
pub const LIST_FOLLOW_RELATIONS_QUERY: &str = r#"
query ListFollowRelations(
    $userid: ID
    $contentFilterBy: ContentFilterType
    $offset: Int
    $limit: Int
) {
    listFollowRelations(
        userid: $userid
        contentFilterBy: $contentFilterBy
        offset: $offset
        limit: $limit
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            followers {
                userid
                username
                slug
                img
                visibilityStatus
                isHiddenForUsers
                hasActiveReports
                isfollowed
                isfollowing
            }
            following {
                userid
                username
                slug
                img
                visibilityStatus
                isHiddenForUsers
                hasActiveReports
                isfollowed
                isfollowing
            }
        }
    }
}
"#;

/// Query: List mutual follows (peers/friends).
pub const LIST_FRIENDS_QUERY: &str = r#"
query ListFriends(
    $userid: ID
    $contentFilterBy: ContentFilterType
    $offset: Int
    $limit: Int
) {
    listFriends(
        userid: $userid
        contentFilterBy: $contentFilterBy
        offset: $offset
        limit: $limit
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            userid
            img
            username
            slug
            biography
            visibilityStatus
            isHiddenForUsers
            hasActiveReports
        }
    }
}
"#;

/// Query: List posts for a specific user (profile posts feed).
pub const LIST_USER_POSTS_QUERY: &str = r#"
query ListUserPosts(
    $userid: ID!
    $filterBy: [PostFilterType!]
    $contentFilterBy: ContentFilterType
    $sortBy: PostSortType
    $offset: Int
    $limit: Int
) {
    listPosts(
        userid: $userid
        filterBy: $filterBy
        contentFilterBy: $contentFilterBy
        sortBy: $sortBy
        offset: $offset
        limit: $limit
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            id
            contenttype
            title
            media
            cover
            mediadescription
            createdat
            amountlikes
            amountviews
            amountcomments
            amountdislikes
            amounttrending
            isliked
            isviewed
            isdisliked
            issaved
            isreported
            tags
            hasActiveReports
            visibilityStatus
            isHiddenForUsers
            user {
                id
                username
                slug
                img
                isfollowed
                isfollowing
                isfriend
            }
        }
    }
}
"#;

/// Mutation: Toggle follow status for a user.
pub const TOGGLE_FOLLOW_MUTATION: &str = r#"
mutation ToggleUserFollowStatus($userid: ID!) {
    toggleUserFollowStatus(userid: $userid) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        isfollowing
    }
}
"#;

/// Mutation: Toggle block status for a user.
pub const TOGGLE_BLOCK_MUTATION: &str = r#"
mutation ToggleBlockUserStatus($userid: ID!) {
    toggleBlockUserStatus(userid: $userid) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
    }
}
"#;

/// Mutation: Report a user.
pub const REPORT_USER_MUTATION: &str = r#"
mutation ReportUser($userid: ID!) {
    reportUser(userid: $userid) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
    }
}
"#;

/// Wrapper for the `getProfile` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProfileData {
    pub get_profile: crate::models::profile::ProfileResponse,
}

/// Wrapper for the `listFollowRelations` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFollowRelationsData {
    pub list_follow_relations: crate::models::profile::FollowRelationsResponse,
}

/// Wrapper for the `listFriends` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFriendsData {
    pub list_friends: crate::models::profile::FriendsResponse,
}

/// Wrapper for the `toggleUserFollowStatus` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleFollowData {
    pub toggle_user_follow_status: crate::models::profile::FollowStatusResponse,
}

/// Wrapper for the `toggleBlockUserStatus` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleBlockData {
    pub toggle_block_user_status: crate::models::profile::GenericMutationResponse,
}

/// Wrapper for the `reportUser` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportUserData {
    pub report_user: crate::models::profile::GenericMutationResponse,
}

// ============================================================================
// Post Creation queries and mutations
// ============================================================================

/// Query: Get post eligibility token.
pub const POST_ELIGIBILITY_QUERY: &str = r#"
query PostEligibility {
    postEligibility {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        eligibilityToken
    }
}
"#;

/// Mutation: Create a new post.
pub const CREATE_POST_MUTATION: &str = r#"
mutation CreatePost($action: PostType!, $input: PostInput!) {
    createPost(action: $action, input: $input) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            contenttype
            title
        }
    }
}
"#;

/// Query: Search tags by name.
pub const SEARCH_TAGS_QUERY: &str = r#"
query SearchTags($tagName: String!, $offset: Int, $limit: Int) {
    searchTags(tagName: $tagName, offset: $offset, limit: $limit) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        counter
        affectedRows {
            name
        }
    }
}
"#;

/// Wrapper for the `postEligibility` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEligibilityData {
    pub post_eligibility: crate::models::post::PostEligibilityResponse,
}

/// Wrapper for the `createPost` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostData {
    pub create_post: crate::models::post::CreatePostResponse,
}

/// Wrapper for the `searchTags` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTagsData {
    pub search_tags: crate::models::post::TagSearchResponse,
}

// ============================================================================
// Chat queries and mutations
// ============================================================================

/// Query: List all chats for the current user.
pub const LIST_CHATS_QUERY: &str = r#"
query ListChats($limit: Int, $offset: Int) {
    listChats(limit: $limit, offset: $offset) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            image
            name
            createdat
            updatedat
            chatmessages {
                id
                senderid
                chatid
                content
                createdat
            }
            chatparticipants {
                userid
                img
                username
                slug
                hasaccess
            }
        }
    }
}
"#;

/// Mutation: Send a message to a chat.
pub const SEND_CHAT_MESSAGE_MUTATION: &str = r#"
mutation SendChatMessage($chatid: ID!, $content: String!) {
    sendChatMessage(chatid: $chatid, content: $content) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            senderid
            chatid
            content
            createdat
        }
    }
}
"#;

/// Mutation: Create a new chat (private or group).
pub const CREATE_CHAT_MUTATION: &str = r#"
mutation CreateChat($name: String!, $recipients: [String!]!, $image: String) {
    createChat(input: { name: $name, recipients: $recipients, image: $image }) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            chatid
        }
    }
}
"#;

/// Wrapper for the `listChats` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListChatsData {
    pub list_chats: crate::models::chat::ListChatsResponse,
}

/// Wrapper for the `sendChatMessage` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendChatMessageData {
    pub send_chat_message: crate::models::chat::SendMessageResponse,
}

/// Wrapper for the `createChat` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatData {
    pub create_chat: crate::models::chat::CreateChatResponse,
}

// ============================================================================
// Wallet queries and mutations
// ============================================================================

/// Query: Get current user's token balance.
pub const BALANCE_QUERY: &str = r#"
query Balance {
    balance {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        currentliquidity
    }
}
"#;

/// Query: Get transaction history with pagination.
pub const TRANSACTION_HISTORY_QUERY: &str = r#"
query TransactionHistory($offset: Int, $limit: Int) {
    transactionHistory(offset: $offset, limit: $limit) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            transactionId
            operationid
            transactionCategory
            transactiontype
            tokenamount
            netTokenAmount
            message
            createdat
            sender {
                userid
                img
                username
                slug
                visibilityStatus
                hasActiveReports
                isHiddenForUsers
            }
            recipient {
                userid
                img
                username
                slug
                visibilityStatus
                hasActiveReports
                isHiddenForUsers
            }
            fees {
                total
                burn
                peer
                inviter
            }
        }
    }
}
"#;

/// Mutation: Transfer tokens to another user.
pub const TRANSFER_MUTATION: &str = r#"
mutation ResolveTransferV2(
    $recipient: ID!
    $numberoftokens: Decimal!
    $message: String!
) {
    resolveTransferV2(
        recipient: $recipient
        numberoftokens: $numberoftokens
        message: $message
    ) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            tokenSendFormatted
            tokensSubstractedFromWalletFormatted
            createdat
        }
    }
}
"#;

/// Query: Get shop order details for a transaction.
pub const SHOP_ORDER_DETAILS_QUERY: &str = r#"
query ShopOrderDetails($transactionId: String!) {
    shopOrderDetails(transactionId: $transactionId) {
        affectedRows {
            shopOrderId
            shopItemId
            shopItemSpecs {
                size
            }
            deliveryDetails {
                name
                email
                addressline1
                addressline2
                city
                zipcode
                country
            }
        }
    }
}
"#;

/// Wrapper for the `balance` query response.
#[derive(Debug, Deserialize)]
pub struct BalanceData {
    pub balance: crate::models::transaction::BalanceResponse,
}

/// Wrapper for the `transactionHistory` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHistoryData {
    pub transaction_history: crate::models::transaction::TransactionHistoryResponse,
}

/// Wrapper for the `resolveTransferV2` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferData {
    pub resolve_transfer_v2: crate::models::transaction::TransferResponse,
}

/// Wrapper for the `shopOrderDetails` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopOrderDetailsData {
    pub shop_order_details: crate::models::transaction::ShopOrderDetailsResponse,
}

// ============================================================================
// Shop mutations
// ============================================================================

/// Mutation: Place a shop order (purchase with tokens).
pub const PERFORM_SHOP_ORDER_MUTATION: &str = r#"
mutation PerformShopOrder(
    $tokenAmount: String!
    $shopItemId: String!
    $name: String!
    $email: String!
    $addressline1: String!
    $addressline2: String
    $city: String!
    $zipcode: String!
    $country: Country!
    $size: String
) {
    performShopOrder(
        tokenAmount: $tokenAmount
        shopItemId: $shopItemId
        orderDetails: {
            name: $name
            email: $email
            addressline1: $addressline1
            addressline2: $addressline2
            city: $city
            zipcode: $zipcode
            country: $country
            shopItemSpecs: { size: $size }
        }
    ) {
        status
        RequestId
        ResponseCode
        ResponseMessage
    }
}
"#;

/// Wrapper for the `performShopOrder` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformShopOrderData {
    pub perform_shop_order: crate::models::shop::PerformShopOrderResponse,
}

// ============================================================================
// Settings mutations
// ============================================================================

/// Mutation: Update the user's profile image.
pub const UPDATE_PROFILE_IMAGE_MUTATION: &str = r#"
mutation UpdateProfileImage($img: String!) {
    updateProfileImage(img: $img) {
        status
        ResponseCode
    }
}
"#;

/// Mutation: Update the user's biography.
pub const UPDATE_BIO_MUTATION: &str = r#"
mutation UpdateBio($biography: String!) {
    updateBio(biography: $biography) {
        status
        ResponseCode
    }
}
"#;

/// Mutation: Update the user's username (requires password confirmation).
pub const UPDATE_USERNAME_MUTATION: &str = r#"
mutation UpdateUsername($username: String!, $password: String!) {
    updateUsername(username: $username, password: $password) {
        status
        ResponseCode
    }
}
"#;

/// Mutation: Update the user's password.
pub const UPDATE_PASSWORD_MUTATION: &str = r#"
mutation UpdatePassword($password: String!, $expassword: String!) {
    updatePassword(password: $password, expassword: $expassword) {
        status
        ResponseCode
    }
}
"#;

/// Mutation: Update the user's email (requires password confirmation).
pub const UPDATE_EMAIL_MUTATION: &str = r#"
mutation UpdateEmail($email: String!, $password: String!) {
    updateEmail(email: $email, password: $password) {
        status
        ResponseCode
    }
}
"#;

/// Mutation: Update user preferences (content filtering, etc.).
pub const UPDATE_PREFERENCES_MUTATION: &str = r#"
mutation UpdateUserPreferences($userPreferences: UserPreferencesInput) {
    updateUserPreferences(userPreferences: $userPreferences) {
        status
        ResponseCode
        affectedRows {
            contentFilteringSeverityLevel
        }
    }
}
"#;

/// Mutation: Delete (deactivate) the user's account.
pub const DELETE_ACCOUNT_MUTATION: &str = r#"
mutation DeleteAccount($password: String!) {
    deleteAccount(password: $password) {
        status
        ResponseCode
    }
}
"#;

/// Wrapper for the `updateProfileImage` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileImageData {
    pub update_profile_image: crate::models::settings::UpdateResponse,
}

/// Wrapper for the `updateBio` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBioData {
    pub update_bio: crate::models::settings::UpdateResponse,
}

/// Wrapper for the `updateUsername` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameData {
    pub update_username: crate::models::settings::UpdateResponse,
}

/// Wrapper for the `updatePassword` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordData {
    pub update_password: crate::models::settings::UpdateResponse,
}

/// Wrapper for the `updateEmail` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailData {
    pub update_email: crate::models::settings::UpdateResponse,
}

/// Wrapper for the `updateUserPreferences` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreferencesData {
    pub update_user_preferences: crate::models::settings::UserPreferencesUpdateResponse,
}

/// Wrapper for the `deleteAccount` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccountData {
    pub delete_account: crate::models::settings::UpdateResponse,
}

// ============================================================================
// Referral queries
// ============================================================================

/// Query: Get the current user's referral info (UUID and shareable link).
pub const GET_REFERRAL_INFO_QUERY: &str = r#"
query GetReferralInfo {
    getReferralInfo {
        status
        ResponseCode
        referralUuid
        referralLink
    }
}
"#;

/// Query: List referral relationships (who I invited and who invited me).
pub const GET_REFERRAL_LIST_QUERY: &str = r#"
query ReferralList($offset: Int, $limit: Int) {
    referralList(offset: $offset, limit: $limit) {
        status
        counter
        ResponseCode
        affectedRows {
            invitedBy {
                id
                username
                slug
                img
            }
            iInvited {
                id
                username
                slug
                img
            }
        }
    }
}
"#;

/// Wrapper for the `getReferralInfo` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReferralInfoData {
    pub get_referral_info: crate::models::referral::ReferralInfoResponse,
}

/// Wrapper for the `referralList` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReferralListData {
    pub referral_list: crate::models::referral::ReferralListResponse,
}

// ============================================================================
// Advertisement queries and mutations
// ============================================================================

/// Query: Get advertisement history with aggregated stats.
pub const ADVERTISEMENT_HISTORY_QUERY: &str = r#"
query AdvertisementHistory($filter: AdvertisementHistoryFilter, $sort: AdvertisementSort, $offset: Int, $limit: Int) {
    advertisementHistory(filter: $filter, sort: $sort, offset: $offset, limit: $limit) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            stats {
                tokenSpent
                euroSpent
                amountAds
                gemsEarned
                amountLikes
                amountViews
                amountComments
                amountDislikes
                amountReports
            }
            advertisements {
                id
                createdAt
                type
                timeframeStart
                timeframeEnd
                totalTokenCost
                totalEuroCost
                gemsEarned
                amountLikes
                amountViews
                amountComments
                amountDislikes
                amountReports
                post {
                    id
                    contenttype
                    title
                    media
                    cover
                    mediadescription
                    visibilityStatus
                    isHiddenForUsers
                    hasActiveReports
                    isreported
                }
                user {
                    id
                    img
                }
            }
        }
    }
}
"#;

/// Mutation: Create a pinned advertisement for a post.
pub const ADVERTISE_POST_PINNED_MUTATION: &str = r#"
mutation AdvertisePostPinned($postid: ID!, $advertisePlan: AdvertisementPinnedPlan!) {
    advertisePostPinned(postid: $postid, advertisePlan: $advertisePlan) {
        meta {
            status
            RequestId
            ResponseCode
            ResponseMessage
        }
        affectedRows {
            id
            createdAt
            type
            timeframeStart
            timeframeEnd
            totalTokenCost
            totalEuroCost
        }
    }
}
"#;

/// Wrapper for the `advertisementHistory` query response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisementHistoryData {
    pub advertisement_history: crate::models::advertisement::AdHistoryResponse,
}

/// Wrapper for the `advertisePostPinned` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisePostPinnedData {
    pub advertise_post_pinned: crate::models::advertisement::AdvertisePostResponse,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_serialization() {
        #[derive(Serialize)]
        struct TestVars {
            name: String,
        }

        let request = GraphQLRequest::new(
            "query Test($name: String!) { hello(name: $name) }",
            TestVars {
                name: "World".into(),
            },
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""query":"query Test"#));
        assert!(json.contains(r#""variables":{"name":"World"}"#));
        assert!(!json.contains("operationName")); // Should be skipped when None
    }

    #[test]
    fn test_graphql_request_with_operation_name() {
        #[derive(Serialize)]
        struct EmptyVars {}

        let request =
            GraphQLRequest::with_operation("mutation DoThing { thing }", EmptyVars {}, "DoThing");

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""operationName":"DoThing""#));
    }

    #[test]
    fn test_graphql_response_success() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestData {
            value: i32,
        }

        let json = r#"{"data": {"value": 42}}"#;
        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        assert!(!response.has_errors());
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().value, 42);
    }

    #[test]
    fn test_graphql_response_with_errors() {
        #[derive(Debug, Deserialize)]
        struct TestData {
            #[allow(dead_code)]
            value: i32,
        }

        let json = r#"{
            "data": null,
            "errors": [
                {"message": "Something went wrong", "locations": [], "path": []}
            ]
        }"#;

        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        assert!(response.has_errors());
        assert_eq!(response.first_error_message(), Some("Something went wrong"));

        let result = response.into_result();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GraphQL(msg) => assert_eq!(msg, "Something went wrong"),
            _ => panic!("Expected GraphQL error"),
        }
    }

    #[test]
    fn test_graphql_response_empty() {
        #[derive(Debug, Deserialize)]
        struct TestData {}

        let json = r#"{"data": null}"#;
        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        let result = response.into_result();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unexpected(msg) => {
                assert!(msg.contains("neither data nor errors"));
            }
            _ => panic!("Expected Unexpected error"),
        }
    }

    #[test]
    fn test_verify_referral_data_deserialization() {
        let json = r#"{
            "verifyReferralString": {
                "status": "success",
                "ResponseCode": "11011",
                "affectedRows": [{
                    "uid": "abc-123",
                    "username": "testuser",
                    "slug": "12345",
                    "img": null
                }]
            }
        }"#;

        let data: VerifyReferralData = serde_json::from_str(json).unwrap();
        assert_eq!(data.verify_referral_string.status, "success");
        assert_eq!(data.verify_referral_string.response_code, "11011");
        assert!(data.verify_referral_string.affected_rows.is_some());
    }
}
