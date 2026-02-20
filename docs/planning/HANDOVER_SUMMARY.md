# CivicConnect: Complete Handover Package
## From Planning to Development

---

## What You're Receiving

This handover package contains everything needed to begin development of CivicConnect, a gamified political organizing platform. The project is currently in the **PLANNING** phase and ready to transition to **DEVELOPMENT**.

### Package Contents

1. **claude.md** - Your primary instruction manual
   - Complete technical overview
   - Development workflows
   - Architecture patterns
   - Troubleshooting guides

2. **STATE** - Current project status
   - What's been done
   - What's in progress
   - What's next
   - Detailed milestone tracking

3. **ECOSYSTEM** - Technical architecture map
   - Full dependency graph
   - Language-specific ecosystems
   - Infrastructure details
   - Data flow patterns

4. **META.scm** - Machine-readable metadata
   - Project configuration in Scheme
   - Queryable project information
   - Timeline and milestones
   - Values and principles

5. **RSR_COMPLIANCE.md** - Development methodology
   - Regenerative Software Requirements
   - Process guidelines
   - Templates and checklists
   - Anti-patterns to avoid

6. **Enhanced PRD** (attached separately)
   - Complete product requirements
   - User personas
   - Technical specifications
   - Success metrics

---

## Quick Start for Claude Code

### Immediate Context

**What**: Platform for grassroots political organizing with gamification
**Why**: Help movements grow through verified peer-to-peer connections
**Who**: US-based organizers, activists, movement leaders
**When**: Target MVP launch April 2025 (4 months)
**How**: Ada/Rust/Elixir stack, self-hosted infrastructure

### Critical Technologies

**ALWAYS USE:**
- Podman (NOT Docker)
- GitLab (NOT GitHub)
- Ada for core business logic
- Rust for API and crypto
- Elixir for real-time features

**NEVER USE:**
- Python (unless absolutely necessary)
- Docker (use Podman)
- GitHub (use GitLab)
- Cloud-managed services (self-host)

### Your First Tasks

1. **Read claude.md** - This is your instruction manual
2. **Review STATE** - Understand where we are
3. **Scan ECOSYSTEM** - Know what we're building with
4. **Check META.scm** - Machine-readable project config
5. **Study RSR_COMPLIANCE.md** - How we develop

---

## What Makes This Project Special

### Not Just Another App

CivicConnect serves **political organizers** who face:
- Government surveillance
- Employer retaliation
- Doxxing and harassment
- Legal consequences for activism

**Every design decision must consider: "Could this endanger an organizer?"**

### Core Values (Non-Negotiable)

1. **Privacy über alles**: Zero-knowledge location, E2E encryption
2. **Security first**: No shortcuts that compromise user safety
3. **Community-owned**: Platform serves organizers, not investors
4. **Accessible always**: Rural users, screen readers must work

### Technical Philosophy

- **Type safety**: Ada and Rust catch errors at compile time
- **Self-hosted**: Data sovereignty and independence
- **Open-source**: Auditable and community-owned
- **Regenerative**: Code quality improves over time (RSR)

---

## Architecture Overview

### Three-Layer System

```
┌─────────────────────────────────────────┐
│         Elixir/Phoenix (Web/WS)         │  ← Real-time UI
│              Port 4000                  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│          Rust (HTTP API)                │  ← REST endpoints
│              Port 8080                  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│      Ada Core (Business Logic)          │  ← Critical paths
│           (Library, FFI)                │
└─────────────────────────────────────────┘
                    ↓
┌──────────────┬──────────────┬───────────┐
│  PostgreSQL  │    Redis     │   MinIO   │  ← Data layer
│   + PostGIS  │   (Cache)    │ (Storage) │
└──────────────┴──────────────┴───────────┘
```

### Why This Stack?

- **Ada**: Mission-critical logic (user accounts, verification, leveling)
  - Type safety prevents entire classes of bugs
  - SPARK formal verification for security-critical paths
  - Ideal for systems where correctness matters more than speed

- **Rust**: Performance-critical operations (API, crypto, location)
  - Memory safety without garbage collection
  - Excellent cryptography libraries
  - Fast enough for real-time location queries

- **Elixir**: Concurrent, fault-tolerant real-time (chat, WebSocket)
  - Erlang VM handles millions of concurrent connections
  - Built-in supervision for fault tolerance
  - Phoenix Channels for WebSocket made easy

---

## Security & Privacy Architecture

### Core Security Features

1. **Zero-Knowledge Location**
   - Server never stores exact coordinates
   - Client-side geohashing (H3 hexagons)
   - Progressive disclosure (trust-based precision)

2. **End-to-End Encryption**
   - Signal protocol for direct messages
   - Keys never leave user device
   - Forward secrecy + post-compromise security

3. **Decentralized Verification**
   - Organizers sign verifications with ed25519
   - No central approval needed
   - Cryptographic audit trail

4. **Privacy by Default**
   - Pseudonymous profiles allowed
   - "Ghost mode" for sensitive organizing
   - Automatic data retention policies

### Threat Model

**Adversaries**:
- Government surveillance (NSA, local police)
- Employer monitoring (activists getting fired)
- Hostile infiltrators (astroturfing, intelligence gathering)
- Malicious users (doxxing, harassment)

**Defenses**:
- Encryption (E2E for messages, TLS for transport)
- Zero-knowledge architecture (minimize data collection)
- Rate limiting (prevent scraping)
- Slow progression (hard to create fake accounts at scale)
- Community moderation (organizers self-police)

---

## Development Process (RSR)

### Sprint Cycle (2 weeks)

```
Week 1:
  Monday: Sprint planning
  Tue-Fri: Development + daily updates
  
Week 2:
  Mon-Thu: Development + daily updates
  Friday: Sprint review + retrospective
```

### Every Sprint Must Have

- [ ] Working software to demo
- [ ] Tests passing (80%+ coverage)
- [ ] Docs updated (if features changed)
- [ ] Security review (if touching auth/crypto)
- [ ] User feedback collected

### Code Quality Standards

**Before Merging**:
- [ ] All tests pass
- [ ] Linters happy (rustfmt, clippy, credo)
- [ ] One reviewer approval
- [ ] ADR written (if architectural decision)
- [ ] No TODOs without GitLab issue

**Red Flags** (do not merge):
- ❌ Tests skipped or commented out
- ❌ Linter warnings ignored
- ❌ Copy-pasted code (refactor into shared function)
- ❌ Hardcoded secrets or config
- ❌ No error handling

---

## Phase 1 Priorities (Next 4 Months)

### Must-Have (MVP)

1. **User Registration** (Month 1)
   - Email/phone verification
   - Password auth (Argon2)
   - JWT tokens
   - Ada: User accounts module
   - Rust: Auth endpoints

2. **Location System** (Month 1-2)
   - GPS → H3 geohash conversion
   - Privacy-preserving storage
   - Proximity queries (PostGIS)
   - Rust: Location service
   - Ada: Privacy logic

3. **Event Creation** (Month 2)
   - CRUD operations
   - Location-based discovery
   - RSVP system
   - Rust: Event API
   - Elixir: Event UI

4. **Verification System** (Month 2-3)
   - QR code generation (organizer)
   - QR code scanning (attendee)
   - Signature verification (ed25519)
   - Experience points award
   - Ada: Verification logic
   - Rust: Crypto operations

5. **Leveling System** (Month 3)
   - Level progression rules
   - Feature unlocks
   - Reputation decay
   - Ada: Leveling engine

6. **Messaging** (Month 3-4)
   - E2E encryption (Signal protocol)
   - WebSocket delivery
   - Offline queue
   - Read receipts
   - Elixir: Chat server
   - Rust: Crypto library

7. **Beta Testing** (Month 3-4)
   - Recruit 3-5 organizing groups
   - Deploy to staging
   - Collect feedback
   - Iterate based on learnings

### Nice-to-Have (Post-MVP)

- Advanced event analytics
- Mentor matching algorithm
- Achievement badges
- Resource library
- Mobile apps (native Swift/Kotlin)

---

## File Structure to Create

```
civicconnect/
├── README.md                   # Start here
├── CHANGELOG.md               # Track changes
├── LICENSE                    # TBD (AGPLv3 or Apache 2.0)
├── CONTRIBUTING.md            # How to contribute
├── CODE_OF_CONDUCT.md         # Community standards
│
├── backend/
│   ├── ada-core/
│   │   ├── civicconnect.gpr   # GNAT project file
│   │   ├── src/
│   │   │   ├── accounts.ads/adb
│   │   │   ├── leveling.ads/adb
│   │   │   ├── verification.ads/adb
│   │   │   └── crypto.ads/adb
│   │   └── tests/
│   │
│   ├── rust-api/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── api/
│   │   │   ├── crypto/
│   │   │   ├── location/
│   │   │   └── db/
│   │   └── tests/
│   │
│   └── elixir-phoenix/
│       ├── mix.exs
│       ├── lib/
│       │   ├── civic_web/
│       │   └── civic/
│       └── test/
│
├── docs/
│   ├── decisions/             # ADRs
│   ├── api/                   # OpenAPI specs
│   ├── architecture/          # Diagrams
│   └── guides/                # User/admin guides
│
├── infrastructure/
│   ├── kubernetes/
│   ├── ansible/
│   ├── .gitlab-ci.yml
│   └── podman-compose.yml
│
├── scripts/
│   ├── setup_dev.sh           # Local dev setup
│   ├── run_tests.sh           # All tests
│   └── deploy_staging.sh      # Deploy script
│
├── STATE                      # Current status
├── ECOSYSTEM                  # Dependencies
├── META.scm                   # Metadata
├── claude.md                  # Instructions for you
└── RSR_COMPLIANCE.md          # Development process
```

---

## Your First Week: Concrete Steps

### Day 1: Environment Setup

```bash
# 1. Clone (or create) GitLab repository
git init civicconnect
cd civicconnect

# 2. Install toolchains
sudo apt install gnat gprbuild  # Ada
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # Rust
# Elixir via asdf or system package manager

# 3. Set up Podman
sudo apt install podman podman-compose
podman version  # Verify install

# 4. Create initial file structure
mkdir -p backend/{ada-core/src,rust-api/src,elixir-phoenix/lib}
mkdir -p docs/{decisions,api,architecture,guides}
mkdir -p infrastructure/{kubernetes,ansible}
mkdir -p scripts

# 5. Copy handover files
cp claude.md STATE ECOSYSTEM META.scm RSR_COMPLIANCE.md .

# 6. Create README.md
# (See template below)

# 7. Set up GitLab CI
# Create .gitlab-ci.yml (see ECOSYSTEM for template)

# 8. Create podman-compose.yml for local dev
# (PostgreSQL + Redis + MinIO)
```

### Day 2: Database Schema

```bash
# 1. Write ADR-001: Database schema design
# Location: docs/decisions/0001-database-schema.md

# 2. Create PostgreSQL migration (Rust sqlx)
cd backend/rust-api
cargo sqlx migrate add initial_schema

# 3. Write SQL for tables:
#    - users
#    - events
#    - verifications
#    - messages
#    - mentorships
#    - level_progression

# 4. Add PostGIS extension and indexes

# 5. Test migration locally
podman-compose up -d postgres
cargo sqlx migrate run
```

### Day 3: Ada Core - User Accounts

```bash
# 1. Create Ada project structure
cd backend/ada-core
alr init --bin civicconnect

# 2. Write accounts.ads (spec)
# Define: User record, Create_User, Authenticate, etc.

# 3. Write accounts.adb (body)
# Implement user creation, authentication logic

# 4. Write tests (AUnit)
cd tests
# Test user creation, duplicate detection, etc.

# 5. Build and test
alr build
alr test
```

### Day 4: Rust API - Authentication Endpoints

```bash
# 1. Set up Rust project
cd backend/rust-api
cargo init

# 2. Add dependencies to Cargo.toml
# axum, tokio, sqlx, serde, etc.

# 3. Create routes: POST /register, POST /login
# Use sqlx to call database
# Return JWT tokens

# 4. Write tests
cargo test

# 5. Run locally
cargo run
# Test with curl or Postman
```

### Day 5: Documentation & Planning

```bash
# 1. Write ADR-002: Authentication flow

# 2. Update STATE file
# Mark user registration as "IN PROGRESS"

# 3. Create Sprint 1 plan in GitLab
# Issues for each task, assign to milestones

# 4. Set up CI/CD
# .gitlab-ci.yml with build + test stages

# 5. Document setup in README
# How to run locally, how to test
```

---

## README.md Template

```markdown
# CivicConnect

Gamified political organizing platform for grassroots movements.

## Quick Start

### Prerequisites
- GNAT (Ada) >= 12.0
- Rust >= 1.75
- Elixir >= 1.16
- Podman >= 4.0
- PostgreSQL >= 15 with PostGIS

### Local Development

1. **Clone repository**
   ```bash
   git clone https://gitlab.com/civicconnect/platform.git
   cd platform
   ```

2. **Start services**
   ```bash
   podman-compose up -d
   ```

3. **Run database migrations**
   ```bash
   cd backend/rust-api
   cargo sqlx migrate run
   ```

4. **Start API server**
   ```bash
   cargo run
   # Listening on http://localhost:8080
   ```

5. **Start web server**
   ```bash
   cd ../elixir-phoenix
   mix deps.get
   mix phx.server
   # Listening on http://localhost:4000
   ```

### Testing

```bash
# Ada tests
cd backend/ada-core && alr test

# Rust tests
cd backend/rust-api && cargo test

# Elixir tests
cd backend/elixir-phoenix && mix test

# All tests
./scripts/run_tests.sh
```

## Documentation

- [Architecture Overview](docs/architecture/overview.md)
- [API Documentation](docs/api/openapi.yaml)
- [Development Guide](claude.md)
- [RSR Compliance](RSR_COMPLIANCE.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

TBD (AGPLv3 or Apache 2.0)
```

---

## Common Questions & Answers

### Q: Why Ada? Isn't it outdated?

**A**: Ada is perfect for security-critical business logic:
- Strong type system catches bugs at compile time
- SPARK subset allows formal verification
- Used in aerospace, defense, medical devices (proven reliability)
- Not trendy, but extremely solid for mission-critical code

**When to use Ada**: User accounts, verification logic, leveling system
**When NOT to use Ada**: Real-time UI, high-throughput APIs (use Rust/Elixir)

### Q: Why self-host instead of AWS/GCP?

**A**: 
- **Data sovereignty**: No third party can be compelled to hand over data
- **Cost**: Hetzner/OVH are 5-10x cheaper than AWS
- **Trust**: Organizers trust self-hosted more than big tech
- **Learning**: Valuable DevOps skills, transferable to other projects

**Trade-off**: More operational burden, but worth it for this use case.

### Q: Why Podman over Docker?

**A**:
- **Rootless**: Runs without root privileges (more secure)
- **Daemonless**: No persistent background process (simpler)
- **Docker-compatible**: Same commands, drop-in replacement
- **Philosophy**: Aligns with open-source, community-owned values

### Q: What if I don't know Ada/Rust/Elixir?

**A**: 
1. Start with language you know best
2. Read existing code in other languages (learn by example)
3. Each language has excellent docs (see Resources in claude.md)
4. Polyglot codebases are manageable with good boundaries

**Focus areas**:
- Ada: Accounts, leveling, verification (3 modules, ~5K lines)
- Rust: API, crypto, location (bulk of codebase, ~20K lines)
- Elixir: Web UI, chat (moderate, ~10K lines)

### Q: How do I handle merge conflicts across languages?

**A**:
- Keep clear boundaries (Ada = library, Rust = API, Elixir = UI)
- FFI contracts defined in .ads files (Ada) and extern in Rust
- Most conflicts will be in shared files (README, docs, CI config)
- Use separate GitLab repos if monorepo becomes unwieldy

---

## Success Criteria

You'll know you're on track if:

**Week 1**:
- [ ] Dev environment set up (all languages compile)
- [ ] Database running locally (PostgreSQL + PostGIS)
- [ ] First ADR written
- [ ] README.md created

**Week 4**:
- [ ] User registration works end-to-end
- [ ] Tests passing in CI
- [ ] Location system prototyped
- [ ] First sprint review conducted

**Month 2**:
- [ ] Event creation/discovery working
- [ ] QR verification prototype
- [ ] Beta testers recruited
- [ ] 5+ ADRs written

**Month 4** (MVP):
- [ ] All core features implemented
- [ ] Security audit passed
- [ ] Beta testers actively using platform
- [ ] Ready for public launch

---

## When You Need Help

### Decision-Making

**Small decisions** (library choice, variable naming):
→ Use best judgment, document in code comments

**Medium decisions** (API design, database schema):
→ Write ADR, open GitLab issue for review

**Large decisions** (major architecture change, pivot):
→ Write ADR, discuss in team meeting, get consensus

### Stuck on Technical Problem

1. **Search docs** (Ada, Rust, Elixir official docs)
2. **Check ECOSYSTEM** (maybe library already exists)
3. **Open GitLab issue** (describe problem, what you've tried)
4. **Ask community** (Ada forums, Rust Discord, Elixir Slack)

### Unclear Requirements

1. **Check claude.md** (detailed instructions)
2. **Check STATE** (current priorities)
3. **Check PRD** (product requirements)
4. **Open GitLab issue** (ask for clarification)

---

## Final Thoughts

CivicConnect is more than an app—it's **infrastructure for democracy**. Every line of code you write enables organizers to build stronger movements, coordinate more effectively, and create lasting change.

The technical challenges are significant:
- Multi-language codebase
- Cryptography and privacy
- Real-time systems at scale
- Self-hosted infrastructure

But the mission is worth it. Organizers need tools they can trust, that respect their privacy, and that help them win their campaigns.

**Build with care. Build with purpose. Build for organizers.**

---

## Next Steps

1. Read claude.md (your instruction manual)
2. Review STATE (understand current status)
3. Scan ECOSYSTEM (know your tools)
4. Set up dev environment (Day 1 tasks above)
5. Write first ADR (database schema)
6. Start coding (user registration first)

**Let's build something that matters. 🚀**

---

*Questions? Open an issue in GitLab or contact the project lead.*
