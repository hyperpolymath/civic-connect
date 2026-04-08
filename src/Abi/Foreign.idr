-- SPDX-License-Identifier: PMPL-1.0-or-later
||| CIVIC-CONNECT — FFI Bridge Declarations
|||
||| This module defines the formal bridge to the native civic-connect
||| implementation. It ensures that decentralized data streams are 
||| handled with strict type safety at the FFI boundary.

module Abi.Foreign

import Abi.Types
import Abi.Layout

%default total

--------------------------------------------------------------------------------
-- Lifecycle
--------------------------------------------------------------------------------

||| Initializes the civic stream connection engine.
export
%foreign "C:civic_connect_init, libcivic"
prim__init : PrimIO Bits64

||| Safe initialization wrapper.
export
init : IO (Maybe Handle)
init = do
  ptr <- primIO prim__init
  pure (createHandle ptr)

||| Shuts down the engine and terminates active stream connections.
export
%foreign "C:civic_connect_free, libcivic"
prim__free : Bits64 -> PrimIO ()

||| Safe cleanup wrapper.
export
free : Handle -> IO ()
free h = primIO (prim__free (handlePtr h))
