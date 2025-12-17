-- SPDX-License-Identifier: AGPL-3.0-or-later
-- SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
--
-- CivicConnect Leveling Module Implementation

package body Civicconnect.Leveling is

   function Calculate_Level
     (XP                 : Experience_Points;
      Events_Attended    : Natural;
      Events_Organized   : Natural;
      Endorsements       : Natural;
      Mentees_Trained    : Natural) return Level_Number
   is
      Potential_Level : Level_Number := 0;
   begin
      --  Find highest level where XP threshold is met
      for L in reverse Level_Number'Range loop
         if XP >= Level_Thresholds (L) then
            Potential_Level := L;
            exit;
         end if;
      end loop;

      --  Check if requirements are also met
      while Potential_Level > 0 loop
         if Meets_Requirements
              (Potential_Level,
               Events_Attended,
               Events_Organized,
               Endorsements,
               Mentees_Trained)
         then
            return Potential_Level;
         end if;
         Potential_Level := Potential_Level - 1;
      end loop;

      return Potential_Level;
   end Calculate_Level;

   function Award_XP
     (User   : User_ID;
      Amount : Experience_Points;
      Reason : String) return Operation_Result
   is
      pragma Unreferenced (User, Reason);
   begin
      if Amount = 0 then
         return Invalid_Input;
      end if;

      --  TODO: Update user XP in database via Rust FFI
      --  TODO: Check for level-up
      --  TODO: Log progression event

      return Success;
   end Award_XP;

   function Meets_Requirements
     (Target_Level       : Level_Number;
      Events_Attended    : Natural;
      Events_Organized   : Natural;
      Endorsements       : Natural;
      Mentees_Trained    : Natural) return Boolean
   is
      Req : constant Level_Requirements := Requirements (Target_Level);
   begin
      return Events_Attended >= Req.Min_Events_Attended
             and then Events_Organized >= Req.Min_Events_Organized
             and then Endorsements >= Req.Min_Endorsements
             and then Mentees_Trained >= Req.Min_Mentees_Trained;
   end Meets_Requirements;

   function Apply_Decay
     (User          : User_ID;
      Days_Inactive : Natural) return Operation_Result
   is
      pragma Unreferenced (User);
   begin
      if Days_Inactive < Inactivity_Threshold_Days then
         return Success;  -- No decay needed
      end if;

      --  TODO: Get current level
      --  TODO: Reduce level by 1 (minimum 0)
      --  TODO: Log decay event
      --  TODO: Notify user

      return Success;
   end Apply_Decay;

   function Get_Features (Level : Level_Number) return Feature_Set is
   begin
      case Level is
         when 0 =>
            return (Can_Message       => False,
                    Can_Create_Events => False,
                    Can_Mentor        => False,
                    Can_Coordinate    => False,
                    Has_Analytics     => False);
         when 1 =>
            return (Can_Message       => True,
                    Can_Create_Events => False,
                    Can_Mentor        => False,
                    Can_Coordinate    => False,
                    Has_Analytics     => False);
         when 2 =>
            return (Can_Message       => True,
                    Can_Create_Events => True,
                    Can_Mentor        => False,
                    Can_Coordinate    => False,
                    Has_Analytics     => False);
         when 3 =>
            return (Can_Message       => True,
                    Can_Create_Events => True,
                    Can_Mentor        => True,
                    Can_Coordinate    => False,
                    Has_Analytics     => False);
         when 4 =>
            return (Can_Message       => True,
                    Can_Create_Events => True,
                    Can_Mentor        => True,
                    Can_Coordinate    => True,
                    Has_Analytics     => False);
         when 5 =>
            return (Can_Message       => True,
                    Can_Create_Events => True,
                    Can_Mentor        => True,
                    Can_Coordinate    => True,
                    Has_Analytics     => True);
      end case;
   end Get_Features;

end Civicconnect.Leveling;
