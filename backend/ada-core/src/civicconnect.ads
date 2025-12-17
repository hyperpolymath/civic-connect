-- SPDX-License-Identifier: AGPL-3.0-or-later
-- SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
--
-- CivicConnect Root Package
-- Provides common types and utilities for the CivicConnect platform

package Civicconnect is
   pragma Pure;

   --  Version information
   Version_Major : constant := 0;
   Version_Minor : constant := 1;
   Version_Patch : constant := 0;

   function Version_String return String;

   --  Common types
   type User_ID is new Positive;
   type Event_ID is new Positive;
   type Level_Number is range 0 .. 5;

   --  Experience points type
   type Experience_Points is range 0 .. 1_000_000;

   --  Result type for operations that can fail
   type Operation_Result is (Success, Failure, Invalid_Input, Not_Found);

end Civicconnect;
