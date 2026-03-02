;; SPDX-License-Identifier: PMPL-1.0-or-later
(ecosystem
  (metadata
    (version "0.3.0")
    (last-updated "2026-02-14"))
  (project
    (name "indieweb2-bastion")
    (purpose "Multi-chain blockchain IndieWeb platform with sovereign DNS, consent, and policy enforcement")
    (role application-platform))
  (related-projects
    (project (name "indieweb") (relationship "implements-standard") (url "https://indieweb.org"))
    (project (name "surrealdb") (relationship "database-dependency") (url "https://surrealdb.com"))
    (project (name "ethereum") (relationship "blockchain-target") (notes "Provenance anchoring"))
    (project (name "polygon") (relationship "blockchain-target") (notes "L2 provenance"))
    (project (name "internet-computer") (relationship "blockchain-target") (notes "Motoko canisters"))
    (project (name "hypatia") (relationship "ci-integration") (notes "Neurosymbolic CI/CD scanning"))
    (project (name "gitbot-fleet") (relationship "bot-orchestration") (notes "Enrolled: findings submitted"))
    (project (name "echidna") (relationship "security-scanning") (notes "Configured: .echidnabot.toml"))
    (project (name "git-private-farm") (relationship "forge-mirroring") (notes "Enrolled: GitHub + GitLab mirrors"))
    (project (name "verisimdb") (relationship "security-scanning") (notes "Vulnerability similarity database"))
    (project (name "panic-attacker") (relationship "security-scanning") (notes "assail scan: 3 medium findings")))
  (standards
    (standard "Rhodium 0.5" "Compliance framework")
    (standard "CRYPTO-POLICY.adoc" "Post-quantum + classical hybrid cryptographic standard")
    (standard "CURPS" "Consent, User Rights, and Policy Standard")))
