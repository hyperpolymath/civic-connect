-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

||| CIVIC-CONNECT — Formal Proofs
|||
||| This module provides formal verification of the security and logic
||| invariants for the Civic-Connect platform.

module Abi.Proofs

import Abi.Types
import Data.So
import Data.Vect

%default total

--------------------------------------------------------------------------------
-- 1. Access Control Correctness (Least Privilege)
--------------------------------------------------------------------------------

||| Only Admin has ManageUsers privilege.
public export
onlyAdminCanManage : (r : Role) -> HasPrivilege r ManageUsers -> (r = Admin)
onlyAdminCanManage Admin AdminManage = Refl

||| Only Admin has AccessPII privilege.
public export
onlyAdminCanPII : (r : Role) -> HasPrivilege r AccessPII -> (r = Admin)
onlyAdminCanPII Admin AdminPII = Refl

||| Citizen cannot ManageUsers.
public export
citizenCannotManage : HasPrivilege Citizen ManageUsers -> Void
citizenCannotManage CitizenRead impossible
citizenCannotManage CitizenSubmit impossible
citizenCannotManage CitizenVote impossible

--------------------------------------------------------------------------------
-- 2. Audit Trail Completeness
--------------------------------------------------------------------------------

||| Proves that for every audited function, an audit entry must exist.
||| This is built into the `Audited` type definition in Types.idr,
||| which acts as a certificate that a mutation was accompanied by an entry.
public export
extractAuditEntry : {f : state -> state} -> {entry : AuditEntry} -> Audited f entry -> AuditEntry
extractAuditEntry (VerifiedMutation f entry) = entry

--------------------------------------------------------------------------------
-- 3. Citizen Data Privacy
--------------------------------------------------------------------------------

||| Proves that PII access requires a positive consent.
public export
piiAccessRequiresConsent : (id : Identity) -> (c : Consent) -> PIIAccess id c -> So (c.granted == True)
piiAccessRequiresConsent id c (HasConsent c prf) = prf

--------------------------------------------------------------------------------
-- 4. Vote/Petition Integrity
--------------------------------------------------------------------------------

||| A simple model of a voting system.
public export
data VoteValue = For | Against

public export
Eq VoteValue where
  For == For = True
  Against == Against = True
  _ == _ = False

||| A valid vote with voter identity.
public export
record TallyVote where
  constructor MkTallyVote
  voter : Identity
  value : VoteValue

||| Count votes for a specific value.
public export
countVotes : VoteValue -> List TallyVote -> Nat
countVotes v [] = 0
countVotes v (x :: xs) =
  if value x == v
    then 1 + countVotes v xs
    else countVotes v xs

||| Tally integrity: Adding a vote for 'v' increases the count for 'v' by exactly 1.
public export
tallyIncreasesByOne : (v : VoteValue) -> (voter : Identity) -> (votes : List TallyVote) ->
                      countVotes v (MkTallyVote voter v :: votes) = 1 + countVotes v votes
tallyIncreasesByOne For voter votes = Refl
tallyIncreasesByOne Against voter votes = Refl

||| Tally integrity: Adding a vote for 'v1' does NOT increase the count for 'v2' if v1 != v2.
public export
tallyIndependent : (v1, v2 : VoteValue) -> (Not (v1 = v2)) -> (voter : Identity) -> (votes : List TallyVote) ->
                   countVotes v2 (MkTallyVote voter v1 :: votes) = countVotes v2 votes
tallyIndependent For Against prf voter votes = Refl
tallyIndependent Against For prf voter votes = Refl
tallyIndependent For For prf voter votes = void (prf Refl)
tallyIndependent Against Against prf voter votes = void (prf Refl)
