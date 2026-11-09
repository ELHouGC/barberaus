# BarberOS Server

The Rust API behind BarberOS. It handles account registration, password authentication, email verification, Redis-backed sessions, client avatar metadata, and haircut-style preferences.

**Not implemented yet:** barber search, follow endpoints, booking, and shop management. The discovery screen in the client uses local demo data, not this API. See [Known limitations](#known-limitations) for implementation gaps that matter before any production use.

## Contents

- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Technology stack](#technology-stack)
- [Architecture](#architecture)
- [API reference](#api-reference)
- [Authentication and sessions](#authentication-and-sessions)
- [Workflows](#workflows)
- [Database and persistence](#database-and-persistence)
- [External integrations](#external-integrations)
- [Testing](#testing)
- [Known limitations](#known-limitations)

## Quick start

1. From the repository root, create `server/.env` and `server/src/infrastructure/nominatim/.env` from their `.env.example` templates. Configure the required values, including a strong `NOMINATIM_PASSWORD`; see the root [Getting started](../README.md#getting-started) guide.
2. Start PostGIS, Redis, and Nominatim from the repository root:

   ```sh
   docker compose --project-directory . -f server/docker-compose.yml --env-file server/.env up -d
   ```

   The stack creates the application database with PostGIS enabled. The first Nominatim dataset import can take considerable time.
3. Apply every SQL file in `server/migrations/` in filename order. The server does **not** run migrations at startup, and no migration command is defined in `Cargo.toml`.
4. Start the API from `server/`:

   ```sh
   cd server
   cargo run
   ```

5. Check it: `curl http://localhost:8080/health`

The API binds to `0.0.0.0:8080`. `/health` only reports that the process is up. It does not check PostgreSQL or Redis.

The Compose stack exposes the Nominatim HTTP API at `http://localhost:8081`, so it does not conflict with the API's port 8080. Nominatim is not needed for any current API workflow.

## Configuration

The server loads `.env` from its working directory through `dotenvy`.

```dotenv
ENV=development
JWT_SECRET=<at-least-32-random-bytes>
DATABASE_URL=postgres://<user>:<password>@<host>:5432/<database>
REDIS_URL=redis://<host>:6379
FROM_EMAIL=<sender-address>
CLOUDINARY_CLOUD_NAME=<cloud-name>
CLOUDINARY_API_KEY=<api-key>
CLOUDINARY_API_SECRET=<api-secret>
# Required only when ENV=production
RESEND_API_KEY=<api-key>
```

| Variable | Required | Notes |
|---|---|---|
| `ENV` | Yes | `development` prints verification emails to the console. `production` sends them through Resend. |
| `JWT_SECRET` | Yes | At least 32 bytes. |
| `DATABASE_URL`, `REDIS_URL` | Yes | |
| `FROM_EMAIL` | Yes | Required in both environments. |
| Cloudinary variables | Yes | Read at startup. |
| `RESEND_API_KEY` | Production only | |

CORS allows the configured local web and Tauri origins in `src/main.rs`. Production origins need a deliberate configuration change before deployment.

## Technology stack

| Area | Implementation |
|---|---|
| HTTP and async runtime | Axum 0.8, Tokio |
| Middleware | `tower-http` CORS |
| Database access | SQLx 0.8 with PostgreSQL |
| Event sourcing and CQRS | `cqrs-es` 0.5, `postgres-es` 0.5 |
| Cache and session store | Redis with `deadpool-redis` |
| Authentication | `argon2`, `jsonwebtoken` |
| Validation and errors | Explicit DTO and domain validation, `thiserror`, typed application and domain errors |
| Serialization and identifiers | Serde/JSON, UUID, Chrono |
| Email | `resend-rs` in production, console adapter in development |
| Media signing | SHA-256 signature generation for Cloudinary |
| Observability | `tracing`, `tracing-subscriber` |

## Architecture

Request path: **routes → handlers → application services → domain commands and ports → infrastructure adapters**.

```text
src/
├── transport/http/        # Routers, handlers, auth middleware, extractors, cookies, error mapping
├── application/           # AuthApp, ClientProfileApp, ports, use-case services, errors
├── domain/user/           # Aggregates, commands, events
├── infrastructure/        # PostgreSQL repositories, Redis sessions, Argon2, JWT, email, Nominatim (unused)
└── cqrs.rs                # Event stores and view queries
```

### Transport

`src/transport/http/` holds the Axum routers, handlers, bearer-auth middleware, request extractors, cookie handling, and error-to-HTTP mapping. Handlers validate or extract transport data, call an application service, and serialize the response. Routes are mounted under `/api/v1`. `/health` is unprefixed.

Successful endpoints generally return `{ "success": true, "data": ... }`. Failures are `application/problem+json` with a status, code, localization key, and optional field violations. Logout is the exception (see [Known limitations](#known-limitations)).

### Application

`AuthApp` orchestrates registration, login, and verification. `ClientProfileApp` coordinates avatar signatures, avatar commands, style preferences, and an internal follow operation. Services depend on abstractions and domain types, so business logic stays out of route handlers.

### Domain and CQRS

`src/domain/user/` defines:

- `UserAccountAggregate`: registration, email verification, and enabling client or barber profiles.
- `ClientProfileAggregate`: initialization, avatar updates, style-preference additions, and follows.
- Account and client-profile events.

`src/cqrs.rs` creates PostgreSQL event stores and generic view queries. `UnifiedUserView` projects both event streams into identity, client-profile, and barber-profile sections of the `user_views` JSONB table.

An email-verification saga listens for `EmailVerified` and initializes the client profile. It logs profile-initialization failures but does not propagate them.

The aggregates contain more concepts than the exposed API. The follow operation has an application method and a handler, but no registered route.

### Persistence and infrastructure

- `src/infrastructure/persistence/`: PostgreSQL credential and email-verification repositories, and database error mapping.
- `src/infrastructure/redis/`: refresh-session creation, rotation, and termination.
- Also here: the Argon2 adapter, JWT helpers, and email sender adapters.

Cloudinary signing is implemented as an application service.

## API reference

All routes are under `/api/v1` unless noted. "Public" means the route is not behind the bearer-auth middleware. It does not imply rate limiting or abuse protection.

### Health

| Method and path | Auth | Behavior |
|---|---|---|
| `GET /health` (unprefixed) | Public | Plain-text health message. Does not check PostgreSQL or Redis. |

### Authentication and sessions

| Method and path | Auth | Behavior |
|---|---|---|
| `POST /auth/signup` | Public | Body: `email`, `password`, `full_name`, `date_of_birth`, `gender`. Requires a non-empty name and a past date of birth for a minimum age of 13. Inserts credentials, records a registration event, and prepares the verification email. Returns `message` and `user_id`. |
| `POST /auth/login` | Public | Body: `email`, `password`. Returns `access_token`, `token_type`, `user`, and sets an HttpOnly refresh cookie. Invalid credentials are indistinguishable by design. Unverified accounts are rejected. |
| `POST /auth/verify-email` | Public | Body: `user_id`, `code`. A valid code activates the account and opens a session, returning the same payload and cookie as login. |
| `POST /auth/resend-verification` | Public | Body: `user_id`. Uses the stored email. Does not resend while an unexpired pending code exists. The client has no control for this yet. |
| `POST /auth/session/refresh` | Refresh credential | Reads the refresh token from a cookie or the `X-Refresh-Token` header, rotates it, and returns a new access token and cookie. |
| `POST /auth/session/logout` | Bearer token and refresh credential | Blacklists the access token, deletes the Redis refresh entry, and clears the cookie. |

### Client profile and styles

The three routes marked "Bearer" use the `require_auth` middleware.

| Method and path | Auth | Behavior |
|---|---|---|
| `GET /client/avatar/signature` | Bearer | Returns Cloudinary signature metadata for the user's avatar folder. |
| `POST /client/avatar` | Bearer | Body: `avatar_url`, `crop_coordinates` (`x`, `y`, `width`, `height`). Validates the URL against the configured Cloudinary host, cloud, and the user's folder, and checks for positive crop dimensions. Then records an avatar-update command. |
| `PUT /me/style-preferences` | Bearer | Body: `selected_style_photo_ids` (array). Resolves catalog IDs to normalized style tags. Rejects missing, duplicate, or unknown IDs. Returns the selected tags. An empty array is accepted. |
| `GET /style-photos?category={gents,ladies,all}` | Public | Returns six catalog entries for the category. `category` is required. Missing, duplicated, or unsupported values produce validation errors. |

There is no follow endpoint and no barber-search endpoint.

### Examples

```sh
# Sign up
curl -X POST http://localhost:8080/api/v1/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "client@example.com",
    "password": "<password>",
    "full_name": "Test Client",
    "date_of_birth": "1995-04-12",
    "gender": "<gender>"
  }'

# Verify email (in development, read the code from the server console)
curl -X POST http://localhost:8080/api/v1/auth/verify-email \
  -H "Content-Type: application/json" \
  -d '{ "user_id": "<user_id>", "code": "<six-digit-code>" }'

# Authenticated request
curl http://localhost:8080/api/v1/client/avatar/signature \
  -H "Authorization: Bearer <access_token>"

# Public catalog
curl "http://localhost:8080/api/v1/style-photos?category=all"
```

The exact formats accepted for `date_of_birth` and `gender` are defined by the signup DTO. Check them against the code before publishing these examples.

### Errors

Errors are mapped in `src/transport/http/problem.rs` and follow RFC 9457 problem details.

| Status | Meaning |
|---|---|
| 401 | Authentication failure |
| 403, 409 | Authorization or business conflict |
| 422 | Validation failure, with field pointers and localization keys |
| 503 | Dependency outage |
| 500 | Internal error |

The HTTP boundary logs mapped application errors, and some repository methods also log database failures.

Email format and password rules are stricter in the client form than in the server's `SignupRequest::validate`. Do not treat the client checks as a substitute for server-side validation.

## Authentication and sessions

1. **Passwords:** `Argon2PasswordHasher` uses generated salts. CPU-bound hashing runs in `spawn_blocking`, bounded by a semaphore. Login returns one invalid-credentials response for unknown emails and wrong passwords.
2. **Email verification:** a six-digit code is generated, and its SHA-256 digest plus a 24-hour expiry are stored in `email_verifications`. The raw code goes out through the configured email adapter. Verification checks the digest for the requested user, then marks the code consumed and activates `user_credentials` in one database transaction.
3. **Account events:** after activation, `AuthApp` dispatches `VerifyEmail` to the account aggregate. The projection marks the read view verified, and the saga initializes the client profile. This CQRS and saga work is outside the credential-update transaction.
4. **Access token:** a signed JWT with the user UUID (`sub`), token ID (`jti`), active profile label, and a 15-minute expiry. Protected requests decode it and check `jti` against the Redis blacklist.
5. **Refresh session:** Redis maps a refresh-token UUID to user and profile metadata with a seven-day TTL. The raw token is sent in an HttpOnly cookie, and the response body contains only the access token. The extractor also accepts `X-Refresh-Token` for non-browser clients.
6. **Refresh and logout:** rotation reads then deletes the old key and creates a new pair. Logout blacklists the access token until it expires and deletes the refresh key.

Redis failures during the blacklist check fail closed and surface as 503.

## Workflows

### Registration and verification

Signup hashes the password, inserts a disabled credential record, and records `RegisterUser` in the account event stream. If that command fails, the service tries to delete the credential record. It then creates a verification record and sends the email.

A mail-provider failure happens after the credential and event are persisted and does not roll them back. Successful verification activates the credential row in a transaction, dispatches the account event, and opens a Redis-backed session.

### Client profile and avatar

The verification saga initializes the client profile. The signature endpoint derives the Cloudinary destination `avatars/{user_id}`. After the client uploads directly to Cloudinary, `POST /client/avatar` checks:

- HTTPS
- the exact Cloudinary delivery host and cloud
- the image-upload path
- the owner folder
- an allowed file extension
- positive crop dimensions

Only then is an event recorded. The API stores the original image URL. Crop coordinates are metadata. The API does not perform an image transformation.

### Style catalog and preferences

The server catalog maps stable photo IDs to semantic style tags. The authenticated preferences command rejects duplicate or unknown IDs, resolves the tags, appends new preferences as a domain event, and returns the projected list. The frontend selection UI does not call this path yet.

### Barber follows

The client aggregate enforces: profile initialized, not self, not already following. The application service also checks that the target has an initialized barber profile in the unified view. No route exposes this, and the demo UI only toggles local state.

## Database and persistence

### Schema

| Table | Purpose |
|---|---|
| `user_credentials` | Identity, password hash, activation and profile flags, timestamps |
| `user_shop_memberships` | Membership and role records, unique per user and tenant |
| `email_verifications` | Code digests, expiry, consumption time, user foreign key |
| `events` | CQRS event envelopes keyed by aggregate type, ID, and sequence |
| `user_views` | JSONB projected views with indexes for identity, city, styles, followed barbers, and barber fields |

The working tree also contains `20261107194740_barber_location.sql`, which adds PostGIS `locations` and barber-location assignment tables. No service or route reads or writes them yet.

The initial migration defines a relational `refresh_tokens` table, but sessions use Redis. Do not treat that table as the live store.

### Migrations and transactions

Migrations are plain SQL in `migrations/`, ordered by filename. Apply them explicitly before starting the API.

Account activation updates `email_verifications` and `user_credentials` in one transaction. CQRS event writes and email delivery are separate operations.

## External integrations

**Cloudinary.** `CloudinaryService` reads its credentials at startup and signs the user-specific folder plus a timestamp. The client uploads directly and then submits the returned secure URL and crop metadata. The API validates the URL structure and owner path before persisting. It never proxies image bytes.

**Email.** Development uses `ConsoleEmailSender`, which prints the message to server output. Production uses `ResendEmailSender` with `RESEND_API_KEY`. Only plain-text verification emails are sent.

**Location.** A Nominatim Compose definition and a PostGIS schema exist. No Rust geocoding adapter, location endpoint, or link from client coordinates to discovery is implemented.

## Testing

```sh
cd server
cargo test
```

Tests live next to the modules they cover. They exercise:

- avatar URL policy and Cloudinary signature generation
- style catalog parsing and ID resolution
- aggregate state transitions and replay
- unified-view projection
- refresh-token extraction
- route validation
- unauthenticated access to protected preferences

Some route tests use a lazy PostgreSQL pool, so this is not a full database-backed integration suite. There is no dedicated test directory and no frontend test suite.

One style-preference projection test currently fails because duplicate style values appear in the projected view; the other 36 server tests pass.

## Known limitations

These are implementation gaps, mostly from development-stage choices. They are not production-ready session security.

| Area | Limitation | Possible fix |
|---|---|---|
| Refresh rotation | Uses separate `GET` and `DEL`, so concurrent use of one refresh token is not consumed atomically. | Single atomic operation (for example `GETDEL`). |
| Refresh cookie | Not marked `Secure`. Creation and clearing use different `SameSite` attributes. | Set both from configuration by environment, and make them match. |
| Verification saga | Client-profile initialization failures are logged, not propagated or retried. A verified account can be left without a profile. | Idempotent retry, or initialize on demand. |
| Registration | A mail-provider failure leaves a persisted account without a delivered code. | Rely on `resend-verification`, or add an outbox. |
| Validation | Server-side email and password rules are weaker than the client's. | Enforce the same rules in `SignupRequest::validate`. |
| Logout response | Returns a plain JSON string, not the standard success envelope. | Return the envelope. |
| Health check | `/health` does not check PostgreSQL or Redis. | Add a readiness endpoint. |
| Schema | The unused `refresh_tokens` table and unused location tables can mislead readers. | Remove or mark as planned. |
| Migrations | Applied manually. | Add a migration runner. |