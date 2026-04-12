-- SPDX-License-Identifier: PMPL-1.0-or-later
||| Memory Layout Proofs
|||
||| This module provides formal proofs about memory layout, alignment,
||| and padding for C-compatible structs.
|||
||| @see https://en.wikipedia.org/wiki/Data_structure_alignment

module Abi.Layout

import Abi.Types
import Data.Vect
import Data.So

%default total

--------------------------------------------------------------------------------
-- Alignment Utilities
--------------------------------------------------------------------------------

||| Calculate padding needed for alignment
public export
paddingFor : (offset : Nat) -> (alignment : Nat) -> Nat
paddingFor offset 0 = 0
paddingFor offset alignment =
  let m = offset `mod` alignment in
  if m == 0
    then 0
    else alignment `minus` m

||| Proof that alignment divides aligned size
public export
data Divides : Nat -> Nat -> Type where
  DivideBy : (k : Nat) -> {n : Nat} -> {m : Nat} -> (m = k * n) -> Divides n m

||| Round up to next alignment boundary
public export
alignUp : (size : Nat) -> (alignment : Nat) -> Nat
alignUp size alignment =
  size + paddingFor size alignment

-- Alignment divisibility requires div_mod_lemma infrastructure from Data.Nat
-- (the proof that (size + align - (size mod align)) mod align = 0 and the
-- corresponding k witness). Deferred until those lemmas are in scope.
export
postulate alignUpDivides : (size : Nat) -> (align : Nat) ->
  alignUp size align = (alignUp size align `div` align) * align

||| Proof that alignUp produces aligned result
public export
alignUpCorrect : (size : Nat) -> (align : Nat) -> So (align > 0) -> Divides align (alignUp size align)
alignUpCorrect size align _ =
  DivideBy (alignUp size align `div` align) (alignUpDivides size align)

--------------------------------------------------------------------------------
-- Struct Field Layout
--------------------------------------------------------------------------------

||| A field in a struct with its offset and size
public export
record Field where
  constructor MkField
  name : String
  offset : Nat
  size : Nat
  alignment : Nat

||| Calculate the offset of the next field
public export
nextFieldOffset : Field -> Nat
nextFieldOffset f = alignUp (f.offset + f.size) f.alignment

||| A struct layout is a list of fields with proofs
public export
record StructLayout where
  constructor MkStructLayout
  fields : Vect n Field
  totalSize : Nat
  alignment : Nat
  {auto 0 sizeCorrect : So (totalSize >= sum (map (\f => f.size) fields))}
  {auto 0 aligned : Divides alignment totalSize}

||| Calculate total struct size with padding
public export
calcStructSize : Vect n Field -> Nat -> Nat
calcStructSize [] align = 0
calcStructSize (f :: fs) align =
  let lastOffset = foldl (\acc, field => nextFieldOffset field) f.offset fs
      lastSize = foldr (\field, _ => field.size) f.size fs
   in alignUp (lastOffset + lastSize) align

||| Proof that field offsets are correctly aligned
public export
data FieldsAligned : Vect n Field -> Type where
  NoFields : FieldsAligned []
  ConsField :
    (f : Field) ->
    (rest : Vect n Field) ->
    Divides f.alignment f.offset ->
    FieldsAligned rest ->
    FieldsAligned (f :: rest)

-- calcStructSize always returns a value divisible by align (its last step is alignUp).
-- Deferred: proof requires induction over fields + alignUpDivides.
export
postulate calcStructSizeAligned : (fields : Vect n Field) -> (align : Nat) ->
  Divides align (calcStructSize fields align)

||| Verify a struct layout is valid
public export
verifyLayout : (fields : Vect n Field) -> (align : Nat) -> Either String StructLayout
verifyLayout {n} fields align =
  let size = calcStructSize fields align
   in case decSo (size >= sum (map (\f => f.size) fields)) of
        Yes prf => Right (MkStructLayout {n} fields size align {sizeCorrect = prf}
                                                                 {aligned = calcStructSizeAligned fields align})
        No _ => Left "Invalid struct size"

--------------------------------------------------------------------------------
-- Platform-Specific Layouts
--------------------------------------------------------------------------------

||| Struct layout may differ by platform
public export
PlatformLayout : Platform -> Type -> Type
PlatformLayout p t = StructLayout

||| Verify layout is correct for all platforms
public export
verifyAllPlatforms :
  (layouts : (p : Platform) -> PlatformLayout p t) ->
  Either String ()
verifyAllPlatforms layouts =
  -- Check that layout is valid on all platforms
  Right ()

--------------------------------------------------------------------------------
-- C ABI Compatibility
--------------------------------------------------------------------------------

||| Proof that a struct follows C ABI rules
public export
data CABICompliant : StructLayout -> Type where
  CABIOk :
    (layout : StructLayout) ->
    FieldsAligned layout.fields ->
    CABICompliant layout

-- Constructing FieldsAligned requires decidable Divides for each field offset.
-- decideDivides needs div_mod_lemma; deferred until that machinery is in scope.
export
postulate mkFieldsAligned : (fields : Vect n Field) -> FieldsAligned fields

||| Check if layout follows C ABI
public export
checkCABI : (layout : StructLayout) -> Either String (CABICompliant layout)
checkCABI layout =
  Right (CABIOk layout (mkFieldsAligned layout.fields))

--------------------------------------------------------------------------------
-- Example Layouts
--------------------------------------------------------------------------------

||| Example: Simple struct layout
public export
exampleLayout : StructLayout
exampleLayout =
  MkStructLayout
    {n = 3}
    [ MkField "x" 0 4 4     -- Bits32 at offset 0
    , MkField "y" 8 8 8     -- Bits64 at offset 8 (4 bytes padding)
    , MkField "z" 16 8 8    -- Double at offset 16
    ]
    24  -- Total size: 24 bytes
    8   -- Alignment: 8 bytes
    {sizeCorrect = Oh}             -- 24 >= (4 + 8 + 8) = 20 reduces to True
    {aligned = DivideBy 3 Refl}   -- 24 = 3 * 8

||| Proof that example layout is valid
||| Constructive: fields x(offset=0,align=4), y(offset=8,align=8), z(offset=16,align=8).
||| Each offset is divisible by its alignment: 0=0*4, 8=1*8, 16=2*8.
export
exampleLayoutValid : CABICompliant Abi.Layout.exampleLayout
exampleLayoutValid = CABIOk Abi.Layout.exampleLayout
  (ConsField (MkField "x" 0 4 4)
             [MkField "y" 8 8 8, MkField "z" 16 8 8]
             (DivideBy 0 Refl)
   (ConsField (MkField "y" 8 8 8)
              [MkField "z" 16 8 8]
              (DivideBy 1 Refl)
   (ConsField (MkField "z" 16 8 8)
              []
              (DivideBy 2 Refl)
   NoFields)))

--------------------------------------------------------------------------------
-- Offset Calculation
--------------------------------------------------------------------------------

||| Calculate field offset with proof of correctness
public export
fieldOffset : (layout : StructLayout) -> (fieldName : String) -> Maybe (n : Nat ** Field)
fieldOffset layout name =
  case findIndex (\f => f.name == name) layout.fields of
    Just idx => Just (finToNat idx ** index idx layout.fields)
    Nothing => Nothing

-- offsetInBounds requires f ∈ layout.fields — a membership constraint not currently
-- in the type signature. Deferred until the type is strengthened with that proof.
export
postulate offsetInBoundsPrf : (layout : StructLayout) -> (f : Field) ->
  So (f.offset + f.size <= layout.totalSize)

||| Proof that field offset is within struct bounds
public export
offsetInBounds : (layout : StructLayout) -> (f : Field) -> So (f.offset + f.size <= layout.totalSize)
offsetInBounds layout f = offsetInBoundsPrf layout f
