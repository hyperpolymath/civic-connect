-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

||| CIVIC-CONNECT — ABI Type Definitions
|||
||| This module defines the Application Binary Interface for the Civic-Connect
||| platform. It ensures that decentralized civic stream data is handled 
||| with type safety across verified language boundaries.

module Abi.Types

import Data.Bits
import Data.So
import Data.Vect

%default total
--------------------------------------------------------------------------------
-- Platform Context
--------------------------------------------------------------------------------

||| Supported targets for civic connectivity modules.
public export
data Platform = Linux | Windows | MacOS | BSD | WASM

||| Resolves the execution environment at compile time.
public export
thisPlatform : Platform
thisPlatform = Linux

--------------------------------------------------------------------------------
-- Core Result Types
--------------------------------------------------------------------------------

||| Formal outcome of a civic-connect operation.
public export
data Result : Type where
  ||| Operation Successful
  Ok : Result
  ||| Operation Failed: Generic error
  Error : Result
  ||| Invalid Parameter: malformed civic data
  InvalidParam : Result
  ||| System Error: out of memory
  OutOfMemory : Result
  ||| Safety Error: null pointer encountered
  NullPointer : Result

--------------------------------------------------------------------------------
-- Identity and Access Control
--------------------------------------------------------------------------------

||| Unique identifier for an entity in the system.
public export
data Identity = MkIdentity String

||| Roles available in the civic-connect platform.
public export
data Role = Citizen | Representative | Admin

||| Privileges that can be granted to roles.
public export
data Privilege = ReadPublicData | SubmitPetition | Vote | AccessPII | ManageUsers

||| Proof that a role has a specific privilege.
public export
data HasPrivilege : Role -> Privilege -> Type where
  CitizenRead      : HasPrivilege Citizen ReadPublicData
  CitizenSubmit    : HasPrivilege Citizen SubmitPetition
  CitizenVote      : HasPrivilege Citizen Vote
  RepRead          : HasPrivilege Representative ReadPublicData
  RepSubmit        : HasPrivilege Representative SubmitPetition
  RepVote          : HasPrivilege Representative Vote
  AdminRead        : HasPrivilege Admin ReadPublicData
  AdminManage      : HasPrivilege Admin ManageUsers
  AdminPII         : HasPrivilege Admin AccessPII

--------------------------------------------------------------------------------
-- Consent and Privacy
--------------------------------------------------------------------------------

||| Explicit consent for data processing.
public export
record Consent where
  constructor MkConsent
  citizen : Identity
  purpose : String
  granted : Bool

||| Proof that PII can only be accessed with valid consent.
public export
data PIIAccess : Identity -> Consent -> Type where
  HasConsent : (c : Consent) -> So (c.granted == True) -> PIIAccess ident c

--------------------------------------------------------------------------------
-- Audit and State
--------------------------------------------------------------------------------

||| An entry in the append-only audit trail.
public export
record AuditEntry where
  constructor MkAuditEntry
  timestamp : Bits64
  actor : Identity
  operation : String
  resource : String

||| Representation of a state change that MUST be audited.
public export
data Audited : (state -> state) -> AuditEntry -> Type where
  VerifiedMutation : (f : state -> state) -> (entry : AuditEntry) -> Audited f entry

--------------------------------------------------------------------------------
-- Safety Handles
--------------------------------------------------------------------------------

||| Opaque handle to a Civic Stream session.
||| INVARIANT: The internal pointer is guaranteed to be non-null.
public export
data Handle : Type where
  MkHandle : (ptr : Bits64) -> {auto 0 nonNull : So (ptr /= 0)} -> Handle

||| Access the underlying pointer of a handle.
public export
handlePtr : Handle -> Bits64
handlePtr (MkHandle ptr) = ptr

||| Safe constructor for civic-connect handles.
public export
createHandle : Bits64 -> Maybe Handle
createHandle ptr =
  case decSo (ptr /= 0) of
    Yes prf => Just (MkHandle ptr {nonNull = prf})
    No _ => Nothing
