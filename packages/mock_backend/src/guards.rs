use async_graphql::{Context, ErrorExtensions, Guard, Result};

/// Role bitmask constants.
pub const ROLE_USER: u32 = 0;
pub const ROLE_SYSTEM: u32 = 1;
pub const ROLE_COMPANY: u32 = 2;
pub const ROLE_BURN: u32 = 4;
pub const ROLE_WEB3_BRIDGE: u32 = 8;
pub const ROLE_ADMIN: u32 = 16;
pub const ROLE_PEER_SHOP: u32 = 32;
pub const ROLE_MODERATOR: u32 = 256;

/// Wrapper for the user's roles bitmask, inserted into context by auth middleware.
pub struct UserRolesMask(pub u32);

/// Guard that checks whether the authenticated user has the required role bitmask.
pub struct RoleGuard {
    pub required_role: u32,
}

impl RoleGuard {
    pub fn new(required_role: u32) -> Self {
        Self { required_role }
    }
}

impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        // First check authentication
        let roles_mask = ctx
            .data_opt::<UserRolesMask>()
            .ok_or_else(|| {
                async_graphql::Error::new("Not authenticated")
                    .extend_with(|_, e| e.set("code", "60501"))
            })?
            .0;

        // Check role
        if self.required_role == ROLE_USER || (roles_mask & self.required_role) != 0 {
            Ok(())
        } else {
            Err(async_graphql::Error::new("Not authorized")
                .extend_with(|_, e| e.set("code", "62101")))
        }
    }
}

/// Convenience: builds a guard requiring MODERATOR role.
pub fn require_moderator() -> RoleGuard {
    RoleGuard::new(ROLE_MODERATOR)
}

/// Convenience: builds a guard requiring ADMIN role.
pub fn require_admin() -> RoleGuard {
    RoleGuard::new(ROLE_ADMIN)
}
