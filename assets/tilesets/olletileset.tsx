<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="olletileset" tilewidth="16" tileheight="16" tilecount="80" columns="10">
 <image source="../textures/olletileset.png" width="160" height="128"/>
 <tile id="0" probability="0.9"/>
 <tile id="10" probability="0.03">
  <objectgroup draworder="index" id="2">
   <object id="1" x="1" y="5" width="14" height="8"/>
  </objectgroup>
 </tile>
 <tile id="12" type="dirt"/>
 <tile id="20">
  <objectgroup draworder="index" id="2">
   <object id="1" x="1" y="5" width="14" height="8"/>
  </objectgroup>
 </tile>
 <tile id="37">
  <objectgroup draworder="index" id="2">
   <object id="1" x="1" y="4" width="14" height="11"/>
  </objectgroup>
 </tile>
 <wangsets>
  <wangset name="Grass" type="corner" tile="-1">
   <wangcolor name="Grass" color="#ff0000" tile="-1" probability="1"/>
   <wangcolor name="Dirt" color="#00ff00" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="1" wangid="0,1,0,2,0,1,0,1"/>
   <wangtile tileid="2" wangid="0,1,0,2,0,2,0,1"/>
   <wangtile tileid="3" wangid="0,1,0,1,0,2,0,1"/>
   <wangtile tileid="4" wangid="0,2,0,1,0,2,0,2"/>
   <wangtile tileid="5" wangid="0,2,0,2,0,1,0,2"/>
   <wangtile tileid="10" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="11" wangid="0,2,0,2,0,1,0,1"/>
   <wangtile tileid="12" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="13" wangid="0,1,0,1,0,2,0,2"/>
   <wangtile tileid="14" wangid="0,1,0,2,0,2,0,2"/>
   <wangtile tileid="15" wangid="0,2,0,2,0,2,0,1"/>
   <wangtile tileid="21" wangid="0,2,0,1,0,1,0,1"/>
   <wangtile tileid="22" wangid="0,2,0,1,0,1,0,2"/>
   <wangtile tileid="23" wangid="0,1,0,1,0,1,0,2"/>
  </wangset>
 </wangsets>
</tileset>
