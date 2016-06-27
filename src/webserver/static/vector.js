
var map = new ol.Map({
  layers: [],
  target: 'map',
  view: new ol.View({
    center: [0, 0],
    zoom: 0
  })
});

var osmCheckbox = document.getElementById('osmCheckbox');
if (osmCheckbox) {
  var osmLayer = new ol.layer.Tile({
                   source: new ol.source.OSM(),
                   opacity: 0.2
                 })
  osmCheckbox.onchange = function(e) {
    if (osmCheckbox.checked) {
      map.getLayers().insertAt(0, osmLayer);
    } else {
      map.removeLayer(osmLayer);
    }
  };
}

var setCenterFromLayer = true;
var tileJsonUrl = 'http://osm2vectortiles.tileserver.com/v1.json';
function parseHash() {
  var hash = window.location.hash.substr(1);
  if (hash) {
    hash = hash.replace(/%7C/g, '|');
    parts = hash.split('|');
    if (parts.length > 0 && parts[0].length > 0) {
      tileJsonUrl = parts[0] || tileJsonUrl;
    }
    if (parts.length > 3) {
      map.getView().setCenter(ol.proj.fromLonLat(
        [parseFloat(parts[2]), parseFloat(parts[1])]
      ));
      map.getView().setZoom(parseInt(parts[3]));
      setCenterFromLayer = false;
    }
  }
}
parseHash();

map.on('postrender', function(e) {
  var lonLat = ol.proj.toLonLat(map.getView().getCenter());
  window.location.hash = tileJsonUrl + '|' + lonLat[1] + '|' + lonLat[0] + '|' + map.getView().getZoom();
});

function generateColor(str) {
  var rgb = [0, 0, 0];
  for (var i = 0; i < str.length; i++) {
      var v = str.charCodeAt(i);
      rgb[v % 3] = (rgb[i % 3] + (13*(v%13))) % 12;
  }
  var r = 4 + rgb[0];
  var g = 4 + rgb[1];
  var b = 4 + rgb[2];
  r = (r * 16) + r;
  g = (g * 16) + g;
  b = (b * 16) + b;
  return [r,g,b,1];
};

function initLayer(data) {
  var layer;
  var layerList = document.getElementById('layerList');
  var layerStyleMap = {}, layerStyleVisibility = {};
  data['vector_layers'].forEach(function(el) {
    var color = generateColor(el['id']);
    var style = new ol.style.Style({
      fill: new ol.style.Fill({color: color}),
      stroke: new ol.style.Stroke({color: color, width: 1})
    });
    layerStyleMap[el['id']] = style;
    layerStyleVisibility[el['id']] = true;

    var item = document.createElement('div');
    item.innerHTML = '<div style="' +
      'background:rgba(' + color[0] + ',' + color[1] + ',' + color[2] + ',1);' +
      '"></div> ' + el['id'];

    item.addEventListener('click', function(e) {
      layerStyleVisibility[el['id']] = !layerStyleVisibility[el['id']];
      item.className = layerStyleVisibility[el['id']] ? '' : 'hidden';
      layer.changed();
    });
    layerList.appendChild(item);
  });

  layer = new ol.layer.VectorTile({
    preload: Infinity,
    source: new ol.source.VectorTile({
      format: new ol.format.MVT(),
      tileGrid: new ol.tilegrid.createXYZ({
        minZoom: data['minzoom'],
        maxZoom: data['maxzoom']
      }),
      tilePixelRatio: 16,
      urls: data['tiles']
    }),
    //extent: ol.proj.transformExtent(data['bounds'], 'EPSG:4326', 'EPSG:3857'),
    style: function(feature, resolution) {
      var layerId = feature.get('layer');
      if (!layerStyleVisibility[layerId]) return null;
      var style = layerStyleMap[layerId];
      // rendering labels on line features does not work in ol3, do not do this
      /*
      if (/_label$/.test(layerId) &&
          layerId != 'road_label' && layerId != 'waterway_label') {
        return [new ol.style.Style({
          text: new ol.style.Text({
            text: feature.get('name') || 'err',
            fill: style.getFill(),
            stroke: new ol.style.Stroke({color: 'rgba(0,0,0,0.8)', width: 1})
          })
        })];
      }
      */
      return [style];
    }
  });

  tileUrlFunction = layer.getSource().getTileUrlFunction();

  if (setCenterFromLayer) {
    var center = data['center'];
    if (typeof center == 'string') {
      center = center.split(',');
    }
    map.getView().setCenter(ol.proj.fromLonLat(
      [parseFloat(center[0]), parseFloat(center[1])]));
    map.getView().setZoom(parseInt(center[2], 10));
  }

  map.addLayer(layer);

  return layer;
}


function loadJson(callback) {
  //var script = document.createElement('script');
  //script.src = tileJsonUrl + '?callback=initLayer'
  //document.getElementsByTagName('head')[0].appendChild(script);

  var xhttp = new XMLHttpRequest();
  xhttp.onreadystatechange = function() {
    if (xhttp.readyState == 4 && xhttp.status == 200) {
      callback(xhttp.response, initLayer(xhttp.response));
    }
  };
  xhttp.responseType = 'json';
  xhttp.open("GET", tileJsonUrl, true);
  xhttp.send();
}
