-- SPDX-License-Identifier: AGPL-3.0-or-later
-- SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
--
-- CivicConnect Root Package Implementation

package body Civicconnect is

   function Version_String return String is
      Major_Str : constant String := Version_Major'Image;
      Minor_Str : constant String := Version_Minor'Image;
      Patch_Str : constant String := Version_Patch'Image;
   begin
      return Major_Str (2 .. Major_Str'Last) & "." &
             Minor_Str (2 .. Minor_Str'Last) & "." &
             Patch_Str (2 .. Patch_Str'Last);
   end Version_String;

end Civicconnect;
