# er-det-noget-værd — TODO

## MVP

- [x] Rust/Axum backend med auth (register, login, email-verifikation, slet konto)
- [x] SQLx migrations (users + reviews)
- [x] `POST /v1/reviews` — opret anmeldelse (titel, beskrivelse, link?, rating 1-5)
- [x] `GET /v1/reviews` — list alle anmeldelser (nyeste først)
- [x] `DELETE /v1/reviews/:id` — slet egen anmeldelse
- [x] Frontend: index.html (login/register, matcher ipfs-apps)
- [x] Frontend: reviews.html (oversigt + opret-formular)
- [x] Docker-compose med Postgres + IPFS
- [x] Dockerfile (non-root, CIS-compliant)
- [x] config.js / config.local.js

## Fremtidige forbedringer

- [ ] Paginering af anmeldelser
- [ ] Filtrér/søg på titel
- [ ] Rediger egen anmeldelse
