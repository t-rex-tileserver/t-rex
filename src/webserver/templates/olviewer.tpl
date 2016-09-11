<!DOCTYPE html>
<html>
  <head>
    <title>T-Rex Vector Tile viewer</title>
    <link rel="stylesheet" href="http://openlayers.org/en/v3.18.2/css/ol.css" type="text/css">
    <script src="http://openlayers.org/en/v3.18.2/build/ol-debug.js"></script>
    <style>
      .map {
        background: #f8f4f0;
      }
    </style>
  </head>
  <body>
    <div id="map" class="map"></div>
    <script>
      var map = new ol.Map({
        layers: [
          new ol.layer.VectorTile({
            source: new ol.source.VectorTile({
              format: new ol.format.MVT(),
              tileGrid: ol.tilegrid.createXYZ({maxZoom: 22}),
              tilePixelRatio: 16,
              url: '{{ baseurl }}/{{ tileset }}/{z}/{x}/{y}.pbf'
            })
          })
        ],
        target: 'map',
        view: new ol.View({
          center: [0, 0],
          zoom: 2
        })
      });
    </script>
  </body>
</html>
