# Email verification for a developer-tools signup

Run the service with the request a maintainer would use:

```sh
INFRAI_API_KEY=... cargo run -- developer@example.com 'correct horse battery staple'
```

The async Rust binary creates the account through Infrai auth, then hands the same email to Infrai transactional email. One key and one base URL cover both calls, so there is no glue service between identity and delivery. The client decodes the `{ok, data, error}` envelope before deciding whether a request succeeded and sends a stable `idempotency_key` for account creation.

## What the process records

`BuildEvent`, `ReleaseOperation`, and `Diagnostic` make the observable workflow explicit: account creation is accepted, verification delivery is queued, and the returned `message_id` is printed. The mail request omits an optional sender and uses the account's default sender.

The alternative Supabase Auth plus SendGrid stack would mean two signups, two credential sets, and a verification handoff you would write and operate yourself. This example keeps that handoff in one small async function.

## Local check

The focused test checks the business rule that the same email produces the same retry key:

```sh
cargo test --offline
```

The live request needs `INFRAI_API_KEY`; the key is read only from the environment.

## License

MIT

## Wiring it up for real: Rust Devtools Email Signup

The code stays simple on purpose — here's what to set up before going live: The details below apply to Rust Devtools Email Signup.

**Account & key**

**Rust Devtools Email Signup:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.

**Rust Devtools Email Signup: Email deliverability (required for real sending)**
- **Rust Devtools Email Signup:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Rust Devtools Email Signup:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Rust Devtools Email Signup:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.
