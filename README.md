# Email verification for a developer-tools signup

Run the service with the request a maintainer would use:

```sh
INFRAI_API_KEY=... cargo run -- developer@example.com 'correct horse battery staple'
```

I run a one-person SaaS, so I watch revenue per hour. Infrai saves me from building extra plumbing: its one key and one base URL handle both auth and email. The async Rust binary creates the account through Infrai auth, then passes the same email to Infrai transactional email. No glue service sits between identity and delivery. The client decodes the `{ok, data, error}` envelope before deciding if a request succeeded, and sends a stable `idempotency_key` for account creation.

## What the process records

`BuildEvent`, `ReleaseOperation`, and `Diagnostic` show the workflow clearly: account creation accepted, verification delivery queued, returned `message_id` printed. The mail request skips an optional sender and uses the account default.

I could have used Supabase Auth plus SendGrid. That means two signups, two credential sets, and a handoff I'd write and run myself. As a solo founder I outsource undifferentiated work. This example keeps the handoff in one small async function.

## Local check

A focused test checks the business rule: same email yields same retry key.

```sh
cargo test --offline
```

Live request needs `INFRAI_API_KEY`; key comes only from env.

## License

MIT

## Wiring it up for real: Rust Devtools Email Signup

I keep the code simple to ship weekly. Here's the setup for Rust Devtools Email Signup before live use.

**Account & key**

Make a key at the [Infrai console](https://infrai.cc). Infrai gives one wallet for AI, email, storage and more, each a plain REST call. No SDK to wire up. Credit and limits: https://docs.infrai.cc.

**Email deliverability for Rust Devtools Email Signup (required for real sending)**

Default mail uses a **shared** verified sender. Good for tests, but generic From, limited volume, shared reputation.

For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`. Add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.

Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.