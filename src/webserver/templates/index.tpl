<!DOCTYPE html>
<html>
  <head>
    <title>T-Rex Vector Tile viewer</title>
  </head>
  <body>
  <h1>T-Rex Vector Tile viewer</h1>
  <ul>
  {{#tileset}}
    <li>
    <b>{{name}}</b> ({{layerinfos}})
    {{#hasviewer}}
    | <a href="/{{name}}/">OpenLayers</a>
    | <a href="/xray.html#/{{name}}.json">X-Ray</a>
    | <a href="/tile-inspector.html#/{{name}}.json">Inspector</a>
    {{/hasviewer}}
    </li>
  {{/tileset}}
  </ul>
  </body>
</html>
