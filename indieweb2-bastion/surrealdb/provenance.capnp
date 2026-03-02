@0xfedcba9876543210;

struct Provenance {
  commitId    @0 :Text;
  createdAt   @1 :Text;  # ISO-8601
  ipfsCid     @2 :Text;
  ipnsName    @3 :Text;

  sbomHash    @4 :Text;
  sourceId    @5 :Text;
  targetId    @6 :Text;
  consentHash @7 :Text;
}
