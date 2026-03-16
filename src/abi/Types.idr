-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

||| CIVIC-CONNECT — ABI Type Definitions
|||
||| This module defines the Application Binary Interface for the Civic-Connect
||| platform. It ensures that decentralized civic stream data is handled 
||| with type safety across verified language boundaries.

module CIVIC_CONNECT.ABI.Types

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
thisPlatform =
  %runElab do
    pure Linux

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
-- Safety Handles
--------------------------------------------------------------------------------

||| Opaque handle to a Civic Stream session.
||| INVARIANT: The internal pointer is guaranteed to be non-null.
public export
data Handle : Type where
  MkHandle : (ptr : Bits64) -> {auto 0 nonNull : So (ptr /= 0)} -> Handle

||| Safe constructor for civic-connect handles.
public export
createHandle : Bits64 -> Maybe Handle
createHandle 0 = Nothing
createHandle ptr = Just (MkHandle ptr)
