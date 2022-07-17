<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.8" tiledversion="1.8.6" name="level_tiles" tilewidth="32" tileheight="32" tilecount="28" columns="14">
 <image source="../tiles/level_tiles_new.png" width="448" height="64"/>
 <tile id="4" type="fry"/>
 <tile id="18" type="spin"/>
 <tile id="22" type="player_end"/>
 <tile id="23" type="player_start"/>
 <tile id="24" type="conveyor">
  <properties>
   <property name="active" value="odd"/>
  </properties>
 </tile>
 <tile id="25" type="conveyor">
  <properties>
   <property name="active" value="even"/>
  </properties>
 </tile>
</tileset>
