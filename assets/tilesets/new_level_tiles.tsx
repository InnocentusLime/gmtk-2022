<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.8" tiledversion="1.9.1" name="new_level_tiles" tilewidth="32" tileheight="32" tilecount="22" columns="1">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="5" type="LogicTile">
  <properties>
   <property name="ty" value="Floor"/>
  </properties>
  <image width="32" height="32" source="../tiles/floor1.png"/>
 </tile>
 <tile id="6" type="LogicTile">
  <properties>
   <property name="ty" value="Floor"/>
  </properties>
  <image width="32" height="32" source="../tiles/floor2.png"/>
 </tile>
 <tile id="7" type="LogicTile">
  <properties>
   <property name="ty" value="Floor"/>
  </properties>
  <image width="32" height="32" source="../tiles/floor3.png"/>
 </tile>
 <tile id="8" type="LogicTile">
  <properties>
   <property name="ty" value="Floor"/>
  </properties>
  <image width="32" height="32" source="../tiles/floor4.png"/>
 </tile>
 <tile id="9" type="LogicTile">
  <properties>
   <property name="ty" propertytype="LevelTileType" value="Start"/>
  </properties>
  <image width="32" height="32" source="../tiles/player_start.png"/>
 </tile>
 <tile id="11" type="LogicTile">
  <properties>
   <property name="ty" value="Exit"/>
  </properties>
  <image width="32" height="32" source="../tiles/exit.png"/>
 </tile>
 <tile id="12" type="LogicTile">
  <properties>
   <property name="ty" value="Conveyor"/>
  </properties>
  <image width="32" height="32" source="../tiles/conveyor1.png"/>
 </tile>
 <tile id="13" type="LevelTileAnimation">
  <properties>
   <property name="anim_ty" value="OnOffAnimation"/>
   <property name="target" value="Conveyor"/>
  </properties>
  <image width="32" height="32" source="../tiles/conveyor2.png"/>
  <animation>
   <frame tileid="12" duration="125"/>
   <frame tileid="13" duration="125"/>
   <frame tileid="14" duration="125"/>
   <frame tileid="15" duration="125"/>
  </animation>
 </tile>
 <tile id="14" type="Frame">
  <image width="32" height="32" source="../tiles/conveyor3.png"/>
 </tile>
 <tile id="15" type="Frame">
  <image width="32" height="32" source="../tiles/conveyor4.png"/>
 </tile>
 <tile id="16" type="LogicTile">
  <properties>
   <property name="ty" propertytype="LevelTileType" value="Frier"/>
  </properties>
  <image width="32" height="32" source="../tiles/fry0.png"/>
 </tile>
 <tile id="17" type="LevelTileAnimation">
  <properties>
   <property name="anim_ty" value="OnTransition"/>
   <property name="target" propertytype="LevelTileType" value="Frier"/>
  </properties>
  <image width="32" height="32" source="../tiles/fry1.png"/>
  <animation>
   <frame tileid="16" duration="125"/>
   <frame tileid="17" duration="125"/>
   <frame tileid="18" duration="125"/>
   <frame tileid="19" duration="125"/>
  </animation>
 </tile>
 <tile id="18" type="LevelTileAnimation">
  <properties>
   <property name="anim_ty" value="OffTransition"/>
   <property name="target" propertytype="LevelTileType" value="Frier"/>
  </properties>
  <image width="32" height="32" source="../tiles/fry2.png"/>
  <animation>
   <frame tileid="19" duration="125"/>
   <frame tileid="17" duration="125"/>
   <frame tileid="18" duration="125"/>
   <frame tileid="16" duration="125"/>
  </animation>
 </tile>
 <tile id="19" type="LevelTileAnimation">
  <properties>
   <property name="anim_ty" value="OnAnimation"/>
   <property name="target" propertytype="LevelTileType" value="Frier"/>
  </properties>
  <image width="32" height="32" source="../tiles/fry3.png"/>
  <animation>
   <frame tileid="20" duration="125"/>
   <frame tileid="21" duration="50"/>
   <frame tileid="22" duration="200"/>
   <frame tileid="23" duration="50"/>
   <frame tileid="19" duration="125"/>
  </animation>
 </tile>
 <tile id="20" type="LevelTileAnimation">
  <properties>
   <property name="anim_ty" value="OffAnimation"/>
   <property name="target" propertytype="LevelTileType" value="Frier"/>
  </properties>
  <image width="32" height="32" source="../tiles/fry4.png"/>
  <animation>
   <frame tileid="16" duration="200"/>
  </animation>
 </tile>
 <tile id="21" type="Frame">
  <image width="32" height="32" source="../tiles/fry5.png"/>
 </tile>
 <tile id="22" type="Frame">
  <image width="32" height="32" source="../tiles/fry6.png"/>
 </tile>
 <tile id="23" type="Frame">
  <image width="32" height="32" source="../tiles/fry7.png"/>
 </tile>
 <tile id="24" type="LogicTile">
  <properties>
   <property name="ty" propertytype="LevelTileType" value="Spinner"/>
  </properties>
  <image width="32" height="32" source="../tiles/spin0.png"/>
 </tile>
 <tile id="25" type="LevelTileAnimation">
  <properties>
   <property name="anim_ty" propertytype="TileAnimationType" value="OnOffAnimation"/>
   <property name="target" propertytype="LevelTileType" value="Spinner"/>
  </properties>
  <image width="32" height="32" source="../tiles/spin1.png"/>
  <animation>
   <frame tileid="24" duration="100"/>
   <frame tileid="25" duration="100"/>
   <frame tileid="26" duration="100"/>
   <frame tileid="27" duration="100"/>
  </animation>
 </tile>
 <tile id="26" type="Frame">
  <image width="32" height="32" source="../tiles/spin2.png"/>
 </tile>
 <tile id="27" type="Frame">
  <image width="32" height="32" source="../tiles/spin3.png"/>
 </tile>
</tileset>
