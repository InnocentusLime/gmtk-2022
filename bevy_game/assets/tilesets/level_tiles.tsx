<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.8" tiledversion="1.8.6" name="level_tiles" tilewidth="32" tileheight="32" tilecount="28" columns="14">
 <image source="../tiles/level_tiles_new.png" width="448" height="64"/>
 <tile id="0">
  <animation>
   <frame tileid="0" duration="33"/>
   <frame tileid="1" duration="33"/>
   <frame tileid="2" duration="33"/>
   <frame tileid="3" duration="33"/>
  </animation>
 </tile>
 <tile id="5" type="fry">
  <properties>
   <property name="active" value="odd"/>
   <property name="anim_type" value="switch"/>
   <property name="off_tile" type="int" value="4"/>
   <property name="off_transition" type="int" value="11"/>
   <property name="on_tile" type="int" value="7"/>
   <property name="on_transition" type="int" value="13"/>
  </properties>
 </tile>
 <tile id="6" type="fry">
  <properties>
   <property name="active" value="even"/>
   <property name="anim_type" value="switch"/>
   <property name="off_tile" type="int" value="4"/>
   <property name="off_transition" type="int" value="11"/>
   <property name="on_tile" type="int" value="7"/>
   <property name="on_transition" type="int" value="13"/>
  </properties>
 </tile>
 <tile id="7">
  <animation>
   <frame tileid="7" duration="150"/>
   <frame tileid="8" duration="150"/>
   <frame tileid="9" duration="200"/>
   <frame tileid="10" duration="150"/>
   <frame tileid="11" duration="200"/>
  </animation>
 </tile>
 <tile id="11">
  <animation>
   <frame tileid="11" duration="125"/>
   <frame tileid="12" duration="125"/>
   <frame tileid="13" duration="125"/>
  </animation>
 </tile>
 <tile id="13">
  <animation>
   <frame tileid="13" duration="125"/>
   <frame tileid="12" duration="125"/>
   <frame tileid="11" duration="125"/>
  </animation>
 </tile>
 <tile id="18" type="spin"/>
 <tile id="22" type="player_end"/>
 <tile id="23" type="player_start"/>
 <tile id="24" type="conveyor">
  <properties>
   <property name="active" value="odd"/>
   <property name="anim_type" value="stop"/>
   <property name="on_tile" type="int" value="0"/>
  </properties>
 </tile>
 <tile id="25" type="conveyor">
  <properties>
   <property name="active" value="even"/>
   <property name="anim_type" value="stop"/>
   <property name="on_tile" type="int" value="0"/>
  </properties>
 </tile>
</tileset>
