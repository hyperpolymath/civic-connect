import Time "mo:base/Time";
import Text "mo:base/Text";

actor Provenance {
  stable var snapshots : [Snapshot] = [];

  public type Snapshot = {
    commitId : Text;
    createdAt : Int;
    ipfsCid : ?Text;
    ipnsName : ?Text;
    signature : Text;
  };

  public func append(s : Snapshot) : async Bool {
    snapshots := Array.append(snapshots, [s]);
    true
  };

  public func latest() : async ?Snapshot {
    if (snapshots.size() == 0) { null } else { ?snapshots[snapshots.size() - 1] }
  };

  public func list() : async [Snapshot] { snapshots };
}
