||| CIVIC-CONNECT — FFI Bridge Declarations
|||
||| This module defines the formal bridge to the native civic-connect
||| implementation. It ensures that decentralized data streams are 
||| handled with strict type safety at the FFI boundary.

module CIVIC_CONNECT.ABI.Foreign

import CIVIC_CONNECT.ABI.Types
import CIVIC_CONNECT.ABI.Layout

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
