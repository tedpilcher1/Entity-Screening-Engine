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
