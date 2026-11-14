[out:json][timeout:170];
(
  nwr["railway"~"^(yard|service_station|junction|site|milestone|stop|crossover_station|passing_loop)$"](-25.65,-49.35,-25.30,-48.45);
  nwr["railway:ref"](-25.65,-49.35,-25.30,-48.45);
  nwr["building"="train_station"](-25.65,-49.35,-25.30,-48.45);
  nwr["public_transport"="station"]["train"="yes"](-25.65,-49.35,-25.30,-48.45);
  nwr["name"~"[Ee]sta..o.*([Ff]errovi|Trem)|[Ee]sta..o (de )?(Paranagu|Alexandra|Saquarema|Porto de Cima|Engenheiro Lange|V.u de Noiva|Ro.a Nova|Pinhais|Piraquara|Carvalho)"](-25.65,-49.35,-25.30,-48.45);
);
out body center;
