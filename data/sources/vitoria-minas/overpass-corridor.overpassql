[out:json][timeout:175][maxsize:536870912];
(
  way["railway"="rail"]["usage"~"^(main|branch)$"](-20.45,-44.05,-18.75,-40.25);
  node["railway"~"^(station|halt|stop)$"](-20.45,-44.05,-18.75,-40.25);
);
out body geom;
