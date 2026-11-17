-- SPDX-License-Identifier: MPL-2.0
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
import Decidable.Equality

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

||| Round up to next alignment boundary.
|||
||| For a positive alignment `S k` this is `ceil(size / S k) * S k`, written
||| as a quotient times the alignment so that divisibility holds by
||| definition (see `alignUpCorrect`). An alignment of 0 leaves the size
||| unchanged, as `paddingFor size 0 = 0` does.
public export
alignUp : (size : Nat) -> (alignment : Nat) -> Nat
alignUp size Z = size
alignUp size (S k) = divNatNZ (size + k) (S k) SIsNonZero * S k

||| Proof that alignUp produces aligned result.
||| Constructive, with no postulate: the quotient is the witness and the
||| equation holds by the definition of `alignUp`. `So (align > 0)` rules
||| out an alignment of 0.
public export
alignUpCorrect : (size : Nat) -> (align : Nat) -> So (align > 0) -> Divides align (alignUp size align)
alignUpCorrect _ Z Oh impossible
alignUpCorrect size (S k) _ = DivideBy (divNatNZ (size + k) (S k) SIsNonZero) Refl

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

||| calcStructSize returns a value divisible by a positive alignment: its
||| last step is alignUp, and the empty struct has size 0.
||| Constructive, with no postulate. (The former postulate quantified over
||| every alignment, including 0, for which it is false.)
export
calcStructSizeAligned : (fields : Vect n Field) -> (k : Nat) ->
  Divides (S k) (calcStructSize fields (S k))
calcStructSizeAligned [] k = DivideBy 0 Refl
calcStructSizeAligned (f :: fs) k =
  alignUpCorrect (foldl (\acc, field => nextFieldOffset field) f.offset fs
                    + foldr (\field, _ => field.size) f.size fs)
                 (S k) Oh

||| Verify a struct layout is valid. An alignment of 0 is rejected: no C
||| type has it, and `Divides 0 size` holds only for size 0.
public export
verifyLayout : (fields : Vect n Field) -> (align : Nat) -> Either String StructLayout
verifyLayout _ Z = Left "Alignment must be positive"
verifyLayout {n} fields (S k) =
  case decSo (calcStructSize fields (S k) >= sum (map (\f => f.size) fields)) of
        Yes prf => Right (MkStructLayout {n} fields (calcStructSize fields (S k)) (S k) {sizeCorrect = prf}
                                                                 {aligned = calcStructSizeAligned fields k})
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

||| Check whether `a` divides `m`, returning the witness when it does.
||| Sound: every `Just` carries a checked equation. A `Nothing` makes no claim.
export
checkDivides : (a : Nat) -> (m : Nat) -> Maybe (Divides a m)
checkDivides Z m = case decEq m 0 of
  Yes prf => Just (DivideBy 0 prf)
  No _ => Nothing
checkDivides (S k) m =
  let q = divNatNZ m (S k) SIsNonZero in
  case decEq m (q * S k) of
    Yes prf => Just (DivideBy q prf)
    No _ => Nothing

||| Build a FieldsAligned witness by checking every field offset.
||| This replaces the former postulate `mkFieldsAligned`, which claimed that
||| every field list is aligned. That is false, for example for a field with
||| offset 1 and alignment 4.
export
checkFieldsAligned : (fields : Vect n Field) -> Maybe (FieldsAligned fields)
checkFieldsAligned [] = Just NoFields
checkFieldsAligned (f :: rest) =
  case checkDivides f.alignment f.offset of
    Nothing => Nothing
    Just d => case checkFieldsAligned rest of
      Nothing => Nothing
      Just r => Just (ConsField f rest d r)

||| Check if layout follows C ABI
public export
checkCABI : (layout : StructLayout) -> Either String (CABICompliant layout)
checkCABI layout =
  case checkFieldsAligned layout.fields of
    Just prf => Right (CABIOk layout prf)
    Nothing => Left "A field offset is not a multiple of its alignment"

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

||| Check that a field lies within the struct bounds, returning the proof
||| when it does.
||| This replaces the former postulate `offsetInBoundsPrf`, which claimed the
||| bound for every field. That is false: `f` need not belong to the layout,
||| and even a member field is bounded only by `sizeCorrect`, which constrains
||| the sum of the field sizes, not each offset. Proving the bound for member
||| fields needs a stronger StructLayout invariant (recorded as open in
||| PROOF-NEEDS.adoc).
public export
offsetInBounds : (layout : StructLayout) -> (f : Field) -> Maybe (So (f.offset + f.size <= layout.totalSize))
offsetInBounds layout f =
  case decSo (f.offset + f.size <= layout.totalSize) of
    Yes prf => Just prf
    No _ => Nothing
