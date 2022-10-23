<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.8" tiledversion="1.9.2" name="activator_tiles" tilewidth="32" tileheight="32" tilecount="2" columns="0">
 <grid orientation="orthogonal" width="1" height="1"/>
 <tile id="0" type="TriggerTileBundle">
  <properties>
   <property name="active" propertytype="ActivationCondition" value="Even"/>
  </properties>
  <image width="32" height="32" source="../tiles/activator_even.png"/>
 </tile>
 <tile id="1" type="TriggerTileBundle">
  <properties>
   <property name="active" propertytype="ActivationCondition" value="Odd"/>
  </properties>
  <image width="32" height="32" source="../tiles/activator_odd.png"/>
 </tile>
</tileset>
