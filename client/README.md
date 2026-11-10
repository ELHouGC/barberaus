# Barberaus Client

The Vue 3 application for Barberaus, hosted in a Tauri 2 shell for desktop and mobile. It covers registration, login, email verification, and early client onboarding: avatar setup, haircut inspiration, and location.

> **Status: early-stage.** Authentication, session handling, and avatar upload are connected to the [API](../server/README.md). Style selection, location, and barber discovery are local UI state or demo data. See [Current status](#current-status).

## Contents

- [Current status](#current-status)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Technology stack](#technology-stack)
- [Architecture](#architecture)
- [Key design points](#key-design-points)
- [User workflows](#user-workflows)
- [API integration](#api-integration)
- [Media and native integrations](#media-and-native-integrations)
- [Testing](#testing)
- [Known limitations](#known-limitations)

## Current status

| Capability | Status | Details |
|---|---|---|
| Signup, email verification, login | Integrated | Calls the Rust API. The client never stores passwords or verification codes beyond the form. |
| Session handling | Integrated | In-memory access token, refresh on startup, single-flight refresh on 401 with one retry. |
| Avatar upload | Integrated (write only) | Signed direct upload to Cloudinary, then URL and crop metadata saved to the API. The saved avatar is not loaded back in a later session. |
| Haircut-style preferences | Local only | Selections live in the onboarding store. The backend catalog and save endpoint exist but are not called. |
| Location | Partial | Reads device coordinates and opens platform settings. Nothing is geocoded, persisted, or used for search. |
| Barber discovery and follows | Demo data | Cards, ratings, distances, filters, and follows use local profiles. The screen says results are illustrative. |
| Onboarding completion | Not complete | The final steps are not wired (see [Known limitations](#known-limitations)). |
| Localization | Done | English and French. |

## Quick start

Run everything from `client/`. The API must be running first (see the [server README](../server/README.md)).

```sh
npm install
npm run dev          # builds the local location-settings plugin, then starts Vite on port 5173
```

| Command | What it does |
|---|---|
| `npm run build` | Builds the local plugin, runs `vue-tsc -b`, then `vite build`. |
| `npm run tauri dev` | Starts the Tauri desktop development app. Tauri runs `npm run dev`. |
| `npm run tauri android dev` | Android development app. Needs the Android SDK. |

iOS development needs macOS and Xcode. Android and iOS builds and device behavior have not been verified. Tauri's configured bundle targets do not guarantee that every native plugin supports every platform.

## Configuration

| Variable | Purpose |
|---|---|
| `VITE_API_BASE_URL` | API base URL. Overrides the built-in defaults. |

If it is unset, `src/transport/http.ts` picks a desktop-local development URL, a fixed development-network URL for Android, or a placeholder production URL. Always set `VITE_API_BASE_URL` for anything beyond local development.

There is no checked-in `.env.example` for the client. Do not put Cloudinary secrets in the client. The server generates the upload signature.

## Technology stack

| Responsibility | Technologies |
|---|---|
| UI | Vue 3 single-file components with `<script setup>` |
| Language and checks | TypeScript 6, `vue-tsc` |
| State and navigation | Pinia 4, Vue Router 4 |
| Forms and validation | vee-validate 4, Yup |
| Localization | Vue I18n 11 (English, French) |
| Styling | Tailwind CSS 4 through `@tailwindcss/vite`, plus project CSS |
| Build | Vite 8 and the Vue Vite plugin |
| Native shell | Tauri 2 |
| Native features | Dialog, filesystem, HTTP, camera, mobile geolocation, and a local location-settings plugin |
| Media | Direct signed uploads to Cloudinary |

Axios is in `package.json` but is not used. The transport is Fetch in the browser and Tauri's HTTP plugin in the native shell.

## Architecture

```text
src/
├── main.ts          # Creates the app and Pinia, tries a session refresh, then mounts
├── router/          # Guest-only auth routes, authenticated client routes, onboarding steps
├── views/auth/      # Signup, login, email verification
├── views/client/    # Placeholder feed and onboarding screens
├── components/      # Shared auth, layout, avatar, and discovery UI
├── composables/     # Avatar upload, camera and gallery, device location, API error mapping, discovery filters
├── stores/          # auth.ts, onboarding.store.ts
├── transport/       # HTTP client, API wrappers, contracts, Cloudinary upload
└── i18n/            # English and French message bundles
```

**Stores**
- `auth.ts` holds the in-memory access token and user payload, and the login, verification, refresh, and logout actions.
- `onboarding.store.ts` holds the avatar draft, selected style-photo IDs, city and coordinates, search radius, and demo follows. It is client state only. It is not persisted and not synced to the backend.

**Routing.** The guard checks whether the auth store has an access token. It does not fetch a profile or enforce onboarding completion.

**Localization.** The locale comes from a saved `localStorage` preference or the browser language, with English as the fallback. Auth and API error messages use translation keys.

## Key design points

| Decision | Reason | Trade-off |
|---|---|---|
| Access token in memory only | Not readable from persistent storage. | The session is lost on reload, so the app must refresh on startup. |
| HttpOnly refresh cookie, `credentials` enabled | The refresh token is not reachable from JavaScript. | Needs CORS and cookie settings to match the server. |
| Single-flight refresh on non-auth 401, one retry | Concurrent callers share one refresh request, which avoids refresh storms. | Auth endpoints are excluded from the retry path. |
| Typed `ApiResponse<T>` instead of thrown HTTP errors | Views and stores branch on the result. Network, invalid-response, and server problem errors are normalized. | Every call site must handle the failure branch. |
| Crop coordinates sent as metadata | The original image is uploaded once. No server-side image processing. | Rendering the crop is the consumer's job. |
| Direct upload to Cloudinary with a server-issued signature | No secrets in the client. Image bytes bypass the API. | Depends on server-side URL and owner validation. |

## User workflows

### Signup, verification, and login

Signup validates the form with Yup and vee-validate and calls `POST /auth/signup`. The user is redirected to a six-digit verification form that calls `POST /auth/verify-email`. On success, a session opens and the user goes to onboarding. Login calls `POST /auth/login`. If the server reports an unverified account, the client can route the user to verification.

The server owns password hashing, code verification, account activation, and session creation.

### Session handling

On startup the client calls `POST /auth/session/refresh`. Requests send credentials and attach the access token as a bearer token. On a 401 from a non-auth endpoint, the client shares one refresh request across concurrent callers and retries the original request once. Logout calls `POST /auth/session/logout` and clears local session state whether or not the request succeeds.

### Avatar onboarding

1. The user picks a gallery image or captures a photo with the native camera.
2. The cropper supports dragging and zooming and computes crop coordinates in the source image's coordinate space. It also creates a local preview.
3. On continue, the client requests a signature from `GET /client/avatar/signature`.
4. It uploads the original image directly to Cloudinary.
5. It submits the secure URL and crop coordinates to `POST /client/avatar`.
6. The server validates the URL structure and folder ownership before saving.

### Haircut-style preferences

The style screen shows inspiration images from a local catalog and keeps the selected IDs in `onboarding.store.ts`. The UI requires three selections to continue. It does not call `GET /style-photos` or `PUT /me/style-preferences`.

### Location and discovery

The location screen can request device coordinates and ask the platform to open location settings. Typing a city updates local state and does not geocode it. Coordinates, city, and radius are not sent to the backend.

Cards, profile sheets, ratings, distances, services, availability, and filters come from local demo profiles. Follow and unfollow only change Pinia state.

## API integration

`src/transport/http.ts` is the shared client. It serializes JSON, attaches the bearer token, enables cookie transport, parses the server's success envelope, and normalizes errors.

| Wrapper | Routes called |
|---|---|
| `auth.api.ts` | Signup, resend-verification, verify-email, login, session refresh, session logout |
| `profile.api.ts` | Avatar signature, avatar metadata save |
| `cloudinary.api.ts` | Direct multipart upload to Cloudinary using the server signature |

Successful responses use `{ "success": true, "data": ... }`. Failures use problem details with a status, code, localization key, and optional field violations. Logout is a current exception and returns a JSON string.

No wrapper calls the style-preference API. A WebSocket transport prototype with a fixed URL exists but is not part of any route or workflow.

## Media and native integrations

**Camera and gallery.** Gallery selection uses the Tauri dialog and filesystem plugins. Camera capture uses `tauri-plugin-camera-kessdev`. The captured image can arrive as a blob, a data URL, or a temporary file path, and the client converts it to a `File` before the shared validate, crop, and upload steps.

The client validator accepts JPEG, PNG, and WebP up to 5 MB. This is a usability check. It does not replace Cloudinary's handling or the server's URL and ownership validation.

**Location.** `@tauri-apps/plugin-geolocation` is registered for mobile and exposed through mobile capability permissions. The location composable checks and requests permission and reads coordinates. The local `tauri-plugin-location-settings` plugin opens the system location settings on Android and the app's settings page on iOS. Neither enables location automatically. Desktop geolocation behavior is not established by the current plugin registration.

**Tauri configuration.** `src-tauri/tauri.conf.json` sets the bundle, development URL, build commands, and content security policy. `src-tauri/capabilities/` lists the default-window plugin permissions and a mobile geolocation capability. Source configuration alone does not show that a platform build or device flow works.

## Testing

`package.json` defines no test, lint, or standalone type-check script. `npm run build` includes `vue-tsc -b`, so it is the only automated check available. There is no frontend test suite.

## Known limitations

| Area | Limitation | Possible fix |
|---|---|---|
| Onboarding flow | The location screen emits `next`, but the parent has no listener. The flow also refers to a barber route that is not registered, so the continue action is a dead end. | Attach the listener and register the route, or remove the action. |
| Style preferences | Not connected to the backend catalog or save endpoint. | Add a `style.api.ts` wrapper and call both endpoints. |
| Discovery and follows | Local demo only. | Needs a server barber-list route and a mounted follow handler. |
| Location | Not persisted, geocoded, or used for search. | Persist coordinates, then add geocoding or a search route. |
| Profile restore | Saved avatar and profile are not loaded after login. | Add a profile read endpoint and load it after refresh. |
| Resend verification | The wrapper exists, but there is no UI control. | Add a control with a cooldown. |
| Route guard | Checks only that a token exists. | Check onboarding state after profile restore. |
| Tooling | No lint, test, or type-check script. Unused Axios dependency and WebSocket prototype. | Add a `typecheck` script, then remove dead code. |
| Configuration | No client `.env.example`. | Add one. |
| Features | No booking or shop management. The feed is a placeholder, and onboarding completion is not saved. | Later milestones. |