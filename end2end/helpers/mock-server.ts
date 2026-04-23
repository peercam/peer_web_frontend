const MOCK_BACKEND_URL = "http://localhost:4000";

/**
 * Reset mock backend state (registered emails, verified users).
 * Call this in `beforeEach` to ensure test isolation.
 */
export async function resetMockState(): Promise<void> {
  const res = await fetch(`${MOCK_BACKEND_URL}/reset`, { method: "POST" });
  if (!res.ok) {
    throw new Error(`Failed to reset mock state: ${res.status} ${res.statusText}`);
  }
}

/**
 * Pre-register an email in the mock backend so subsequent registration
 * attempts with this email return a duplicate error (30601).
 *
 * Uses the GraphQL register mutation directly.
 */
export async function preRegisterEmail(email: string): Promise<void> {
  const query = `
    mutation {
      register(input: {
        email: "${email}",
        password: "TestPass123!",
        username: "preregistered_user",
        referralUuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93"
      }) {
        status
        ResponseCode
      }
    }
  `;

  const res = await fetch(`${MOCK_BACKEND_URL}/graphql`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });

  if (!res.ok) {
    throw new Error(`Failed to pre-register email: ${res.status}`);
  }
}

/**
 * Fetch the most recent password-reset token issued for `email` from the
 * mock backend's debug endpoint. Used to drive the multi-step forgot-password
 * flow without an SMTP transport.
 */
export async function getResetTokenForEmail(email: string): Promise<string> {
  const url = `${MOCK_BACKEND_URL}/debug/reset-token?email=${encodeURIComponent(email)}`;
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(
      `Failed to fetch reset token for ${email}: ${res.status} ${res.statusText}`,
    );
  }
  const body = (await res.json()) as { token?: string };
  if (!body.token) {
    throw new Error(`No reset token returned for ${email}`);
  }
  return body.token;
}
