-- SPDX-License-Identifier: AGPL-3.0-or-later
-- SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
--
-- CivicConnect Leveling Module
-- Gamification system with exponential progression
--
-- Design principles:
-- - Exponential requirements (easy to start, hard to master)
-- - Real-world verification required (can't solo grind)
-- - Mentorship gates (must help others to advance)
-- - Quality over quantity

package Civicconnect.Leveling is

   --  Level thresholds (exponential)
   --  Level 0: Newcomer (0 XP)
   --  Level 1: Attendee (100 XP)
   --  Level 2: Active Member (500 XP)
   --  Level 3: Organizer (1500 XP)
   --  Level 4: Senior Organizer (4000 XP)
   --  Level 5: Movement Leader (10000 XP)

   type Level_Threshold_Array is array (Level_Number) of Experience_Points;

   Level_Thresholds : constant Level_Threshold_Array :=
     (0 => 0,
      1 => 100,
      2 => 500,
      3 => 1_500,
      4 => 4_000,
      5 => 10_000);

   --  XP awards for various activities
   XP_Event_Attendance    : constant Experience_Points := 25;
   XP_First_Event         : constant Experience_Points := 50;  -- Bonus
   XP_Event_Organized     : constant Experience_Points := 100;
   XP_Mentee_Leveled_Up   : constant Experience_Points := 75;
   XP_Endorsement_Given   : constant Experience_Points := 10;
   XP_Endorsement_Received : constant Experience_Points := 15;

   --  Reputation decay: Lose 1 level after 6 months of inactivity
   Inactivity_Threshold_Days : constant := 180;
   Decay_Check_Interval_Days : constant := 30;

   --  Level requirements beyond XP
   type Level_Requirements is record
      Min_Events_Attended   : Natural;
      Min_Events_Organized  : Natural;
      Min_Endorsements      : Natural;
      Min_Mentees_Trained   : Natural;
   end record;

   Requirements : constant array (Level_Number) of Level_Requirements :=
     (0 => (0, 0, 0, 0),
      1 => (1, 0, 0, 0),        -- Attend 1 event
      2 => (3, 0, 1, 0),        -- 3 events + 1 endorsement
      3 => (5, 2, 3, 0),        -- 5 events + 2 organized + 3 endorsements
      4 => (10, 5, 5, 3),       -- 10 events + 5 organized + 5 endorsements + 3 mentees
      5 => (20, 10, 10, 5));    -- 20 events + 10 organized + 10 endorsements + 5 mentees

   --  Calculate current level from XP and activity
   function Calculate_Level
     (XP                 : Experience_Points;
      Events_Attended    : Natural;
      Events_Organized   : Natural;
      Endorsements       : Natural;
      Mentees_Trained    : Natural) return Level_Number;

   --  Award XP and check for level-up
   function Award_XP
     (User    : User_ID;
      Amount  : Experience_Points;
      Reason  : String) return Operation_Result;

   --  Check if user meets requirements for a level
   function Meets_Requirements
     (Target_Level       : Level_Number;
      Events_Attended    : Natural;
      Events_Organized   : Natural;
      Endorsements       : Natural;
      Mentees_Trained    : Natural) return Boolean;

   --  Apply reputation decay for inactive users
   function Apply_Decay
     (User          : User_ID;
      Days_Inactive : Natural) return Operation_Result;

   --  Get feature unlocks for a level
   type Feature_Set is record
      Can_Message        : Boolean;
      Can_Create_Events  : Boolean;
      Can_Mentor         : Boolean;
      Can_Coordinate     : Boolean;
      Has_Analytics      : Boolean;
   end record;

   function Get_Features (Level : Level_Number) return Feature_Set;

end Civicconnect.Leveling;
