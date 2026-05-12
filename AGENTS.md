@../adrs/README.md
@../guidelines/README.md

---

## Projektbeskrivelse

Trustpilot-lignende anmeldelsesservice til brug blandt venner. Brugere kan oprette anmeldelser med titel, beskrivelse (emoji-støtte), valgfrit link og en rating fra 1–5.

## Stack

- **Backend:** Rust, Axum 0.7, SQLx 0.8 + PostgreSQL
- **Frontend:** Vanilla JS — ingen framework, hostet via IPFS
- **Auth:** Argon2id, JWT HS256, Cloudflare Turnstile, email-verifikation via Resend
- **Deploy:** Docker Compose, Caddy, Ansible

## Arkitektur

```
Frontend (IPFS)  →  er-det-noget-vaerd.apps.gihc.online  →  Caddy  →  score:8080 (Axum)
                                                                              │
                                                                          PostgreSQL
```

## Kom i gang

```bash
cp frontend/config.local.js frontend/config.js   # brug lokal API-URL og Turnstile test-key
docker compose up --build
```

Lokalt springes Turnstile og Resend over (tom `TURNSTILE_SECRET` / `RESEND_API_KEY`).

## Vigtige filer og moduler

| Fil | Formål |
|-----|--------|
| `src/lib.rs` | AppState, router, CORS-lag |
| `src/auth.rs` | Argon2id, JWT, `authenticate`-helper |
| `src/routes/auth.rs` | Register, login, me, slet konto, email-verifikation |
| `src/routes/reviews.rs` | List, opret, slet anmeldelse |
| `src/models.rs` | `User`, `Review`, `ReviewWithAuthor`, `CreateReview` |
| `frontend/index.html` | Login/register-flow |
| `frontend/reviews.html` | Anmeldelsesoversigt og opret-formular |
| `migrations/0001_initial.sql` | `users`- og `reviews`-tabeller |

## API-oversigt

| Metode | Sti | Auth |
|--------|-----|------|
| POST | `/v1/auth/register` | — |
| POST | `/v1/auth/token` | — |
| GET | `/v1/auth/me` | Bearer |
| DELETE | `/v1/auth/me` | Bearer |
| GET | `/v1/auth/verify?token=` | — |
| GET | `/v1/reviews` | — |
| POST | `/v1/reviews` | Bearer |
| DELETE | `/v1/reviews/:id` | Bearer |

## Nøglebeslutninger

| ADR | Beslutning |
|-----|------------|
| [0001](../adrs/0001-axum-over-actix.md) | Axum over Actix-web |
| [0002](../adrs/0002-sqlx-runtime-api.md) | SQLx runtime API, ikke compile-time makroer |
| [0003](../adrs/0003-rustls-over-native-tls.md) | rustls over native-tls |
| [0004](../adrs/0004-lib-bin-split.md) | Lib + bin-split for testbarhed |
| [0005](../adrs/0005-authenticate-plain-async-fn.md) | `authenticate` som plain async fn |
| [0008](../adrs/0008-ipfs-dnslink-frontend.md) | IPFS + DNSLink til frontend |
| [0011](../adrs/0011-argon2id-passwords.md) | Argon2id til password-hashing |
| [0012](../adrs/0012-feature-flags-via-empty-env.md) | Tom env-variabel som feature flag |

## Deploy

Deploy kræver `ansible-galaxy collection install -r ansible/requirements.yml` første gang.

```bash
# Fuld deploy: test → beta → prod
ansible-playbook ansible/deploy.yml -i ansible/inventory.yml --ask-vault-pass

# Eller trin for trin:
ansible-playbook ansible/deploy-test.yml    -i ansible/inventory.yml --ask-vault-pass
ansible-playbook ansible/deploy-promote.yml -i ansible/inventory.yml --ask-vault-pass
```

Deploy-flowet synkroniserer filer til VPS, bygger Docker-images, opdaterer IPFS-frontenden og DNS via Simply.com API, og kører smoke tests mod hvert miljø. Prod-deploy efterfølges af et OWASP ZAP baseline-scan.

Hemmeligheder ligger i `ansible/group_vars/all/vault.yml` (krypteret med ansible-vault, gitignored).

**Miljøer og domæner:**

| Miljø | API | Frontend |
|-------|-----|----------|
| prod | `score.api.gihc.online` | `er-det-noget-vaerd.apps.gihc.online` |
| beta | `beta.score.api.gihc.online` | `beta.er-det-noget-vaerd.apps.gihc.online` |
| test | `test.score.api.gihc.online` | `test.er-det-noget-vaerd.apps.gihc.online` |

## Projekt-specifikke detaljer

- Integration tests kræver en superuser-forbindelse *uden* databasenavn: `DATABASE_URL=postgres://postgres:password@localhost:5432 cargo test`. `#[sqlx::test]` opretter en frisk database pr. test og dropper den bagefter.
- `frontend/config.js` indeholder prod-URL og Turnstile site key — må ikke committes med lokal konfiguration.
