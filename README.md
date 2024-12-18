# Entity Screening Engine


## To do:
- (Low priority) Create new service to handle tracking tasks/jobs, recording duration, completion, metrics etc (could be used for costing too)
- (High priority) Create service to handle risk calculation: find sanctions, criminal records, shell company analysis etc
-- OpenSanctions: Peps, Regulatory watchlists, Sanctioned Securities, Warrents and Criminal Entities
- (Very low priority) Create service for LLM integration, document summary, risk breakdown etc
- (High priority )React frontend (todo when depressed)
- (Medium priority) Add entity search job, used to get company number based on name, some fuzzy match etc, probably can just use ch api

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

- echo DATABASE_URL=postgres://username:password@localhost/diesel_demo > .env


## Commonly required commands
- diesel migration generate --diff-schema name_of_migration