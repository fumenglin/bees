OUTPUT_PATH = /project1/bin/$(notdir $(CURDIR))

hello:
	echo "Hello"

#build:
#	GOOS=linux CGO_ENABLED=0 go build -o target/main main.go broker.go model.go stat.go config.go
#	cp config.yml target/config.yml
#	cp -r cases target/

#run:
#	go run cdm/cloud/main.go

#clean:
#	rm -rf target/*

bees:
	docker build --tag bees -f Dockerfile .


