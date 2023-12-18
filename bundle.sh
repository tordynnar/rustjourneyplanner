#!/bin/sh
mv dist/*.wasm dist/journey.wasm
mv dist/*.js dist/journey.js
mv dist/*.css dist/journey.css
mv dist/index.html dist/journey.htm
sed -i "s/\/rustjourneyplanner\-.*\.js/journey\.js/g" dist/journey.htm
sed -i "s/\/rustjourneyplanner\-.*\.wasm/journey\.wasm/g" dist/journey.htm
sed -i "s/\/style\-.*\.css/journey\.css/g" dist/journey.htm
zip --junk-paths release.zip dist/*