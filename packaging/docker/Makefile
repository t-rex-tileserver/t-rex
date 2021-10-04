VERSION_TAG = 0.14.1
DEB_URL = https://github.com/t-rex-tileserver/t-rex/releases/download/v$(VERSION_TAG)/t-rex_$(VERSION_TAG)-1.focal_amd64.deb

build:
	docker build --build-arg DEB_URL=$(DEB_URL) -t sourcepole/t-rex -f Dockerfile .
	docker run sourcepole/t-rex --version

push:
	docker login
	docker push sourcepole/t-rex
	docker tag sourcepole/t-rex sourcepole/t-rex:$(VERSION_TAG)
	docker push sourcepole/t-rex:$(VERSION_TAG)

run:
	docker run -p 6767:6767 -v $$PWD/../../data:/var/data/in:ro -v /tmp:/var/data/out sourcepole/t-rex serve --bind=0.0.0.0 --openbrowser=false --datasource=natural_earth.gpkg
