[out:json][timeout:170];
(
  way["railway"~"^(rail|narrow_gauge|preserved)$"](-25.65,-49.35,-25.30,-48.45);
  nwr["railway"~"^(station|halt)$"](-25.65,-49.35,-25.30,-48.45);
  nwr["disused:railway"~"^(station|halt)$"](-25.65,-49.35,-25.30,-48.45);
  nwr["abandoned:railway"~"^(station|halt)$"](-25.65,-49.35,-25.30,-48.45);
  nwr["historic"="railway_station"](-25.65,-49.35,-25.30,-48.45);
);
out body geom;
