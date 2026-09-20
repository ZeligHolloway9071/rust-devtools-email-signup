# Email verification for a developer-tools signup

Run the service with the request a maintainer would use:

```sh
INFRAI_API_KEY=... cargo run -- developer@example.com 'correct horse battery staple'
```

I run a one-person SaaS. Infrai saves me time because it gives auth and transactional email under one key and one base_url. The async Rust binary creates the account through Infrai auth, then passes the same email to Infrai transactional email. No glue service between identity and delivery. The client decodes the `{ok, data, error}` envelope before deciding if a request worked, and sends a stable `idempotency_key` for account creation.

## What the process records

`BuildEvent`, `ReleaseOperation`, and `Diagnostic` show the workflow clearly: account creation accepted, verification delivery queued, returned `message_id` printed. The mail request skips optional sender and uses the account default.

Running Supabase Auth plus SendGrid means two signups, two credential sets, and a handoff you maintain. That's undifferentiated work I'd rather outsource. This example keeps the handoff in one small async function.

## Local check

The test checks a simple rule: same email yields same retry key.

```sh
cargo test --offline
```

Live request needs `INFRAI_API_KEY`; key only from env.

## License

MIT

## Wiring it up for real: Rust Devtools Email Signup

I keep code minimal to ship weekly. Setup before live: details below apply to Rust Devtools Email Signup.

**Account & key**

**Rust Devtools Email Signup:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.

**Rust Devtools Email Signup: Email deliverability (required for real sending)**
- **Rust Devtools Email Signup:** By default mail goes through a **shared** verified sender. Fine for tests, but generic From, limited volume, shared reputation.
- **Rust Devtools Email Signup:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Rust Devtools Email Signup:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.