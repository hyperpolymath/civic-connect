-- SPDX-License-Identifier: AGPL-3.0-or-later
-- SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
--
-- CivicConnect Accounts Module
-- User account management with privacy-first design
--
-- Security: This module handles user credentials and PII
-- All passwords must be hashed before storage (via Rust FFI to Argon2)
-- No PII should be logged

with Ada.Strings.Unbounded; use Ada.Strings.Unbounded;
with Ada.Calendar;

package Civicconnect.Accounts is

   --  Maximum lengths for validation
   Max_Email_Length    : constant := 254;
   Max_Username_Length : constant := 64;
   Min_Password_Length : constant := 12;

   --  User record (minimal PII)
   type User_Record is record
      ID            : User_ID;
      Email_Hash    : Unbounded_String;  -- SHA256 hash, not plaintext
      Username      : Unbounded_String;  -- Display name (may be pseudonym)
      Current_Level : Level_Number := 0;
      XP            : Experience_Points := 0;
      Location_Hash : Unbounded_String;  -- H3 cell ID, never coordinates
      Created_At    : Ada.Calendar.Time;
      Last_Active   : Ada.Calendar.Time;
      Is_Verified   : Boolean := False;
   end record;

   --  Account operations
   function Create_User
     (Email    : String;
      Username : String;
      Password : String) return Operation_Result;
   --  Create a new user account
   --  Password will be hashed via Rust FFI (Argon2)

   function Authenticate
     (Email    : String;
      Password : String;
      User_Out : out User_Record) return Operation_Result;
   --  Authenticate user, return user record if successful

   function Get_User
     (ID       : User_ID;
      User_Out : out User_Record) return Operation_Result;
   --  Retrieve user by ID

   function Update_Location_Hash
     (ID       : User_ID;
      H3_Cell  : String) return Operation_Result;
   --  Update user's location hash (H3 cell ID)
   --  Never store actual coordinates

   function Validate_Email (Email : String) return Boolean;
   --  Basic email format validation

   function Validate_Password (Password : String) return Boolean;
   --  Password strength validation
   --  Requirements: 12+ chars, mixed case, numbers, symbols

private

   --  Internal validation helpers
   function Is_Valid_H3_Cell (Cell : String) return Boolean;

end Civicconnect.Accounts;
