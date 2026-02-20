# Regenerative Software Requirements (RSR) Compliance Guide
## CivicConnect Implementation

---

## What is RSR?

**Regenerative Software Requirements (RSR)** is a development methodology that treats software as a living ecosystem requiring continuous care, adaptation, and renewal. Unlike traditional "set-and-forget" requirements, RSR emphasizes:

1. **Living Documentation**: Requirements evolve with the project
2. **Iterative Development**: Small cycles, frequent feedback
3. **Sustainability Focus**: Long-term maintainability over short-term velocity
4. **Community Involvement**: Stakeholders co-create the system
5. **Adaptive Requirements**: Flexibility to change based on evidence
6. **Regenerative Practices**: Code health improves over time

RSR is inspired by regenerative agriculture, permaculture, and living systems thinking. The goal is software that becomes more valuable, robust, and adaptable over time—not more brittle and technical debt-laden.

---

## Core RSR Principles for CivicConnect

### 1. Living Documentation

**Traditional Approach**: Write PRD once, code to spec, documentation drifts
**RSR Approach**: Documentation is continuously updated, code and docs stay synchronized

**Implementation**:
- **Architecture Decision Records (ADRs)**: Every significant decision documented
- **Active README**: Updated every sprint with current status
- **Code-level docs**: Explain *why*, not just *what*
- **Decision log**: Track what changed and why

**Example ADR Format**:
```markdown
# ADR-001: Use Zero-Knowledge Location Storage

## Status
Accepted

## Context
Users need to discover nearby events without revealing exact location to server.
Organizers may face retaliation if government can identify them via location data.

## Decision
Use client-side geohashing (H3 hexagons at resolution 7 ≈ 5km cells).
Server stores only cell IDs, never coordinates.
Progressive disclosure: Higher-level users get more precision.

## Consequences
**Positive**:
- Server breach doesn't expose user locations
- Regulatory compliance (no PII location data)
- User trust and safety

**Negative**:
- Location queries slightly less precise
- Client must compute hashes (small CPU cost)
- More complex than storing lat/lon directly

## Alternatives Considered
1. Store encrypted coordinates: Still vulnerable to decryption
2. Store zip codes: Too coarse for urban areas
3. No location storage: Defeats proximity discovery feature

## Reversibility
Medium - would require database migration and client changes, but feasible

## Related ADRs
- ADR-009: Progressive Trust Model
- ADR-010: Decentralized Verification System
```

**Where to Store ADRs**: `/docs/decisions/NNNN-title.md`

**When to Create ADR**:
- Technology choice (database, language, library)
- Architectural pattern (microservices, event-driven, etc.)
- Security/privacy decision
- Major feature trade-off
- Anything you'll need to explain in 6 months

**RSR Compliance Checklist**:
- [ ] ADRs written *before* implementation, not after
- [ ] README updated every sprint
- [ ] Every module has purpose documentation
- [ ] Breaking changes documented in CHANGELOG.md
- [ ] Runbooks exist for operational tasks

---

### 2. Iterative Development

**Traditional Approach**: 6-month waterfall, big reveal at end
**RSR Approach**: 2-week sprints, ship small increments, gather feedback

**Implementation**:
- **Sprint duration**: 14 days
- **Sprint goals**: 1-3 concrete deliverables
- **Daily standups**: 15 minutes, async in GitLab (not Slack)
- **Sprint review**: Demo working software to stakeholders
- **Sprint retro**: What worked? What to change?

**CivicConnect Sprint Cadence**:
```
Week 1: Monday
├── Sprint Planning (2 hours)
│   ├── Review backlog
│   ├── Select stories for sprint
│   ├── Break into tasks
│   └── Assign owners
│
├── Daily Work (Mon-Fri)
│   └── Daily async update in GitLab issue
│
Week 2: Friday
├── Sprint Review (1 hour)
│   └── Demo to beta testers or stakeholders
│
└── Sprint Retro (1 hour)
    ├── What went well?
    ├── What to improve?
    └── Action items for next sprint
```

**Sprint Anti-Patterns to Avoid**:
- ❌ No working software at end of sprint
- ❌ Carrying over same tasks for 3+ sprints (break into smaller tasks)
- ❌ Skipping retros ("too busy")
- ❌ Not involving users/organizers in review

**User Feedback Loops**:
- **Every sprint**: Share progress with 3-5 beta tester organizers
- **Every month**: User interview (20-30 min) with 2 new organizers
- **Every quarter**: Broader survey (50+ users)

**RSR Compliance Checklist**:
- [ ] Sprint planning happens on schedule
- [ ] At least one demo-able feature per sprint
- [ ] Retro action items tracked and reviewed next sprint
- [ ] User feedback collected regularly
- [ ] Backlog groomed weekly

---

### 3. Sustainability Focus

**Traditional Approach**: Ship fast, deal with tech debt later (narrator: they never do)
**RSR Approach**: Sustainable velocity, invest in maintainability

**Implementation**:
- **20% time**: Every sprint, 20% of capacity goes to tech debt, refactoring, tooling
- **Code review standards**: Maintainability > cleverness
- **Test coverage**: Maintain 80%+ coverage, never merge if tests fail
- **Dependency hygiene**: Quarterly dependency audit, monthly security patches
- **Energy efficiency**: Optimize database queries, minimize compute waste

**Sustainability Practices**:

**Code Health**:
- Refactor when complexity exceeds threshold (cyclomatic complexity > 10)
- Extract reusable modules when pattern repeats 3+ times
- Delete unused code aggressively (dead code is technical debt)
- Prefer boring technology (proven > shiny new)

**Team Health**:
- No weekend work (except production emergencies)
- Rotate on-call duties (no single point of burnout)
- Document tribal knowledge (no hero dependency)
- Encourage vacation (rested devs write better code)

**Environmental Impact**:
- Profile CPU/memory usage quarterly
- Optimize hot paths (80/20 rule: fix 20% of code causing 80% of load)
- Use efficient data structures (arrays > linked lists for cache locality)
- Self-hosting reduces cloud compute waste

**RSR Compliance Checklist**:
- [ ] 20% time tracked and protected
- [ ] Tech debt tickets in backlog, prioritized regularly
- [ ] Code complexity metrics monitored (SonarQube, CodeClimate)
- [ ] No merge without passing tests
- [ ] Documentation updated with code changes

---

### 4. Community Involvement

**Traditional Approach**: Build in isolation, hope users like it
**RSR Approach**: Co-create with stakeholders from day one

**Implementation**:
- **Beta tester cohort**: 3-5 organizing groups embedded from Month 1
- **Open development**: Roadmap visible, users can propose features
- **User research**: 20 interviews before starting development
- **Community input**: Features prioritized based on organizer needs
- **Open-source release**: Month 12, community can contribute

**Stakeholder Engagement**:

**Pre-Development** (Before coding):
- [ ] 20+ user interviews with organizers across 5 movements
- [ ] Survey: What tools do you use? What frustrates you?
- [ ] Usability study: Show mockups, get feedback

**During Development** (Months 1-12):
- [ ] Beta tester Slack/Discord (or GitLab issues)
- [ ] Monthly community call (30 min, demo + Q&A)
- [ ] Suggestion box (users submit feature ideas)
- [ ] Voting on features (community prioritizes next work)

**Post-Launch** (Month 12+):
- [ ] Public roadmap (GitLab milestones)
- [ ] Contributor guide (CONTRIBUTING.md)
- [ ] Code of conduct (CODE_OF_CONDUCT.md)
- [ ] Maintainer ladder (how to become core team)

**RSR Compliance Checklist**:
- [ ] User research conducted before major features
- [ ] Beta testers consulted every sprint
- [ ] Feature requests tracked in public backlog
- [ ] Community input influences prioritization
- [ ] Open-source release plan defined

---

### 5. Adaptive Requirements

**Traditional Approach**: "We said we'd build X, so we'll build X even if it's wrong"
**RSR Approach**: "Evidence suggests Y is more valuable than X, let's adapt"

**Implementation**:
- **Hypothesis-driven**: Every feature is a hypothesis to test
- **Metrics-informed**: Track success metrics, pivot if not hitting goals
- **Fail fast**: If feature isn't used, deprecate quickly
- **Flexibility**: Requirements negotiable, values non-negotiable

**CivicConnect Value Hierarchy**:
```
Non-Negotiable (Never compromise):
├── Privacy (zero-knowledge location, E2E encryption)
├── Security (no shortcuts that endanger users)
├── Accessibility (rural users, screen readers must work)
└── Community ownership (platform serves organizers, not investors)

Negotiable (Open to change based on evidence):
├── Feature scope (do we need AR? analytics dashboard?)
├── Gamification details (5 levels or 10? badges or no badges?)
├── UI/UX specifics (map view or list view?)
└── Monetization (premium tier or ads? what features are premium?)
```

**Adaptation Process**:
1. **Hypothesis**: "We believe [feature] will increase [metric]"
2. **Test**: Build MVP, release to beta testers
3. **Measure**: Track metric for 2 sprints
4. **Decide**: Keep, iterate, or kill
5. **Document**: Update ADR with learnings

**Example**:
- **Hypothesis**: "AR event discovery will increase event attendance"
- **Test**: Build AR prototype in Month 11 (pilot in 2 cities)
- **Measure**: Do users use AR? Does it increase attendance vs. map view?
- **Decide**: If usage < 10%, kill feature (too niche, high maintenance)
- **Document**: ADR-042: "Why we removed AR feature"

**RSR Compliance Checklist**:
- [ ] Features framed as hypotheses
- [ ] Success criteria defined before building
- [ ] Metrics tracked for all features
- [ ] Deprecation policy exists (what conditions trigger removal?)
- [ ] Pivots documented with reasoning

---

### 6. Regenerative Practices

**Traditional Approach**: Code quality degrades over time (entropy)
**RSR Approach**: Code health improves over time (regeneration)

**Implementation**:
- **Boy Scout Rule**: Leave code cleaner than you found it
- **Scheduled refactoring**: Every N sprints, pick a module to improve
- **Dependency updates**: Keep dependencies fresh, don't accrue debt
- **Performance profiling**: Regular checks, optimize bottlenecks
- **Security hardening**: Continuous improvement, not one-time audit

**Regenerative Rituals**:

**Weekly**:
- Code review for maintainability (not just correctness)
- Dependency vulnerability scan (automated in CI)
- Delete dead code (grep for unused functions)

**Monthly**:
- Refactor one gnarly module (rotate through codebase)
- Performance profiling (find slowest queries)
- Documentation audit (fix stale docs)

**Quarterly**:
- Dependency upgrade sprint (update all libraries)
- Architecture review (are patterns still serving us?)
- Accessibility audit (test with screen reader)
- Security audit (third-party pen test)

**Example Refactoring Schedule**:
```
Sprint 3: Refactor Ada user account module (simplify auth flow)
Sprint 6: Refactor Rust location service (extract geohashing into library)
Sprint 9: Refactor Elixir chat (reduce coupling, improve tests)
Sprint 12: Major dependency upgrade (PostgreSQL 15 → 16, etc.)
```

**Code Health Metrics** (track in Grafana dashboard):
- Test coverage percentage (target: 80%+)
- Cyclomatic complexity (avg per function, target: < 10)
- Number of TODOs/FIXMEs in codebase (reduce over time)
- Dependency age (avg months since last update, target: < 6)
- Build time (target: < 10 minutes)

**RSR Compliance Checklist**:
- [ ] Boy Scout Rule enforced in code reviews
- [ ] Refactoring time allocated every sprint
- [ ] Metrics dashboard shows improving trends
- [ ] No accumulation of "we'll fix it later" debt
- [ ] Regular audits scheduled and completed

---

## RSR Implementation Roadmap for CivicConnect

### Month 1: Foundation
- [x] Create claude.md, STATE, ECOSYSTEM, META.scm
- [ ] Set up GitLab with ADR template
- [ ] Recruit 3-5 beta tester organizing groups
- [ ] Conduct initial 20 user interviews
- [ ] Establish sprint cadence

### Months 2-4: MVP with RSR Practices
- [ ] Write ADRs for all major decisions
- [ ] 2-week sprint reviews with beta testers
- [ ] Track metrics dashboard (even if manual)
- [ ] 20% time on tooling, CI/CD, tests
- [ ] Monthly retros, adapt process as needed

### Months 5-8: Community Integration
- [ ] Open public roadmap on GitLab
- [ ] Monthly community calls
- [ ] User voting on features
- [ ] Contributor guide drafted
- [ ] First external contribution merged

### Months 9-12: Open Source & Scale
- [ ] Open-source core platform
- [ ] Maintainer ladder published
- [ ] Quarterly security audit
- [ ] Dependency audit and upgrade sprint
- [ ] Year 1 retrospective: What worked? What to change?

---

## RSR Tools & Templates

### ADR Template
See example above. Copy to `/docs/decisions/NNNN-title.md`

### Sprint Planning Template
```markdown
# Sprint N: [Dates]

## Sprint Goal
[One sentence: What's the main focus?]

## Stories
1. [Story title]
   - Tasks: [Task 1], [Task 2], ...
   - Owner: [Name]
   - Estimate: [Hours or story points]

## 20% Time
- [Tech debt ticket]
- [Refactoring task]
- [Tooling improvement]

## User Feedback
- Beta tester demo on [date]
- User interview with [person] on [date]

## Success Criteria
- [ ] All stories demo-able
- [ ] Tests pass
- [ ] Docs updated
```

### Retro Template
```markdown
# Sprint N Retrospective

## What Went Well? ✅
- [Thing 1]
- [Thing 2]

## What Could Be Better? 🔧
- [Thing 1]
- [Thing 2]

## Action Items for Next Sprint 🎯
- [ ] [Action 1] - Owner: [Name]
- [ ] [Action 2] - Owner: [Name]

## Shoutouts 🎉
- Thanks to [person] for [thing]!
```

### User Interview Script
```markdown
# User Interview: [Organizer Name]

## Background (5 min)
- What movement/cause do you organize for?
- How long have you been organizing?
- What tools do you currently use?

## Pain Points (10 min)
- What's most frustrating about organizing?
- What takes the most time?
- What stops you from growing your movement?

## Feature Reaction (10 min)
- [Show mockup/prototype]
- What do you think of [feature]?
- Would you use this? Why or why not?
- What's missing?

## Wrap-up (5 min)
- Any other feedback?
- Can we follow up next month?
```

---

## Common RSR Anti-Patterns (What NOT to Do)

❌ **Writing ADRs after the fact**
→ Decision context is lost, ADR becomes justification instead of reasoning

❌ **Skipping retros when "too busy"**
→ Exactly when you need them most; pressure → mistakes → more pressure

❌ **Ignoring user feedback because "we know better"**
→ Organizers are the experts on organizing, not you

❌ **Accumulating tech debt "until after launch"**
→ Launch never ends, debt compounds, codebase becomes unmaintainable

❌ **Documentation as afterthought**
→ Undocumented code = unmaintainable code = eventual rewrite

❌ **Optimizing for short-term velocity**
→ Burns out team, accrues debt, slows down long-term

❌ **Building features no one asked for**
→ Waste of time, clutters product, confuses users

❌ **Treating requirements as immutable**
→ Rigidity → building the wrong thing → wasted effort

---

## RSR Success Indicators

You're doing RSR right if:

✅ You can onboard a new developer in < 1 week (good docs, clear architecture)
✅ You can explain any major decision by reading an ADR (decisions documented)
✅ Your test suite catches regressions (high-quality tests)
✅ Users say "this feature is exactly what I needed" (co-creation works)
✅ Your codebase is easier to work with now than 6 months ago (regenerative)
✅ You pivot quickly when something isn't working (adaptive)
✅ Team morale is high and sustainable (not burning out)

---

## RSR Metrics Dashboard

Track these in Grafana or similar:

**Process Metrics**:
- ADRs written per month (target: 2+)
- Sprint velocity (story points per sprint)
- Retro action items completed (target: 100%)
- User interviews conducted (target: 2+ per month)

**Code Health**:
- Test coverage % (target: 80%+)
- Build time (target: < 10 min)
- Dependency age (avg months, target: < 6)
- Open GitHub/GitLab issues (should trend down over time)

**Community**:
- Beta tester engagement (active users per month)
- External contributions (PRs from non-core team)
- Community calls attendance

**Adaptation**:
- Features killed/deprecated (healthy to cull unused features)
- Pivots per quarter (some pivots = responsive, too many = unfocused)
- Time from idea to shipped (should decrease over time)

---

## Conclusion: Why RSR Matters for CivicConnect

CivicConnect serves political organizers—people who face retaliation, surveillance, and burnout. The platform must be:

- **Reliable**: Organizers depend on it; downtime = missed opportunities
- **Secure**: Breaches endanger users; no shortcuts on security
- **Adaptable**: Organizing tactics evolve; platform must evolve too
- **Sustainable**: This is a multi-year project; team health matters
- **Community-owned**: Organizers co-create the tools they need

RSR is not "extra process"—it's how you build software that lasts and serves its users well.

**RSR is infrastructure for long-term success.**

Without it, you get:
- Brittle codebase that breaks constantly
- Burned-out team that can't sustain development
- Features no one uses because no one asked for them
- Technical debt that makes changes glacially slow
- Platform that doesn't adapt to user needs

With it, you get:
- Robust codebase that improves over time
- Sustainable team that can iterate for years
- Features that delight users because they co-created them
- Healthy codebase where changes are quick and safe
- Platform that evolves with the organizing landscape

---

**For CivicConnect, RSR is not optional. It's how we honor the organizers we serve.**
