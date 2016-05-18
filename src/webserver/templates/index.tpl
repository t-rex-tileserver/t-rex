<!DOCTYPE html>
<html>
  <head>
    <title>T-Rex Vector Tile viewer</title>
  </head>
  <body>
  <h1>T-Rex Vector Tile viewer</h1>
  <ul>
  {{#layer}}
    <li>
    {{#hasviewer}}
      <a href="/{{name}}/">{{name}}</a>
    {{/hasviewer}}
    {{^hasviewer}}
      {{name}}
    {{/hasviewer}} ({{geomtype}})
    </li>
  {{/layer}}
  </ul>
  </body>
</html>
