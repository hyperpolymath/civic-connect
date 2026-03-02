-- Strict schema for bastion

DEFINE TABLE identities SCHEMAFULL;
DEFINE FIELD publicKey ON TABLE identities TYPE string;
DEFINE FIELD dnsTxt    ON TABLE identities TYPE string;

DEFINE TABLE manifests SCHEMAFULL;
DEFINE FIELD manifests ON TABLE manifests TYPE array;

DEFINE TABLE edges SCHEMAFULL;
DEFINE FIELD edges ON TABLE edges TYPE array;

DEFINE TABLE provenance SCHEMAFULL;
DEFINE FIELD commitId   ON TABLE provenance TYPE string;
DEFINE FIELD createdAt  ON TABLE provenance TYPE datetime;
DEFINE FIELD ipfsCid    ON TABLE provenance TYPE string;
DEFINE FIELD ipnsName   ON TABLE provenance TYPE string;

DEFINE FIELD sbomHash   ON TABLE provenance TYPE string;
DEFINE FIELD sourceId   ON TABLE provenance TYPE string;
DEFINE FIELD targetId   ON TABLE provenance TYPE string;
DEFINE FIELD consentHash ON TABLE provenance TYPE string;
