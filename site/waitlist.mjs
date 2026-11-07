// Signup body for the early-access invitation.
// Marketing consent is not collected here, so the flag stays false
// even if a caller passes a truthy value.

export function buildWaitlistPayload(fields) {
  const source = fields || {};
  return {
    email: String(source.email || "").trim(),
    role: String(source.role || "").trim(),
    company: String(source.company || "").trim(),
    website: String(source.website || ""),
    source: String(source.source || ""),
    purpose: "early_access_invitation",
    marketing_opt_in: false,
  };
}
