import Nat "mo:base/Nat";
import Text "mo:base/Text";

actor Consent {
  stable var versions : [Text] = [];
  stable var store : Trie = Trie.Empty;

  public type Manifest = {
    version : Text;
    telemetry : Text;
    indexing : Text;
    cid : ?Text;
  };

  public type Trie = {
    Empty : ();
    Node  : { key : Text; value : Manifest; next : Trie };
  };

  public func put(key : Text, m : Manifest) : async Bool {
    versions := Array.append(versions, [m.version]);
    store := putTrie(store, key, m);
    true
  };

  public func get(key : Text) : async ?Manifest {
    lookupTrie(store, key)
  };

  public func listVersions() : async [Text] { versions };

  func putTrie(t : Trie, k : Text, v : Manifest) : Trie {
    switch t {
      case (#Empty) { #Node({ key = k; value = v; next = #Empty }) };
      case (#Node(n)) {
        if (n.key == k) { #Node({ key = k; value = v; next = n.next }) }
        else { #Node({ key = n.key; value = n.value; next = putTrie(n.next, k, v) }) }
      }
    }
  };

  func lookupTrie(t : Trie, k : Text) : ?Manifest {
    switch t {
      case (#Empty) { null };
      case (#Node(n)) { if (n.key == k) { ?n.value } else lookupTrie(n.next, k) }
    }
  };
}
