# Screening Engine


## Requirements to run
- Docker running
- Pulsar running (in docker)
`docker run -it \
-p 6650:6650 \
-p 8080:8080 \
--mount source=pulsardata,target=/pulsar/data \
--mount source=pulsarconf,target=/pulsar/conf \
apachepulsar/pulsar:4.0.0 \
bin/pulsar standalone`
- Postgres running

- Pulsar GUI:
`docker pull apachepulsar/pulsar-manager:latest
docker run -it \
  -p 9527:9527 -p 7750:7750 \
  -e SPRING_CONFIGURATION_FILE=/pulsar-manager/pulsar-manager/application.properties \
  apachepulsar/pulsar-manager:latest`