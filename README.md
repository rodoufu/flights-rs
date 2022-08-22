# flights-rs

This project provides a simple API for processing flights.
For the purpose of this solution each request and each flight is independent, so there is no caching by saving information.

## API

### GET /health

This endpoint can be used to check if the service is health.
It could also be used as the readiness probe.

### GET /metrics

This endpoint is answering with Prometheus default format for the registered metrics.

### POST /flight

It expects a JSON like this:
```json
{"legs": [["IND","EWR"],["SFO", "ATL"],["GSO", "IND"],["ATL", "GSO"]],"full_path":true}
```

The field `full_path` is optional and its default value is false, it indicates if the response should or should not have the path.

Considering the server is running in the localhost at 8080, a simple curl request to that endpoint can be done using cURL like this:
```shell
curl -X POST -H 'Content-Type: application/json' localhost:8080/flight -d '{"legs": [["IND","EWR"],["SFO", "ATL"],["GSO", "IND"],["ATL", "GSO"]],"full_path":true}'
```

The expected response is:
```json
{"type":"Ok","source":"SFO","destination":"EWR","path":["SFO","ATL","GSO","IND","EWR"]}
```

## Build

The service can be built with `make build`, it will not require Rust to be installed in the current machine only Docker.
It is going to be created a docker image named `flights-rs:latest`.

One can run the service with `make run` using the default values and creating a container named `flights-rs`.

All the tests are run with `make test`.

## Stress test

It was used the project `hey` https://github.com/rakyll/hey to generate a simple load test on the service:
```shell
NR=100000
NW=100
DATA='{"legs": [["20","21"],["21","22"],["22","23"],["23","24"],["24","25"],["8","9"],["14","15"],["15","16"],["16","17"],["17","18"],["18","19"],["19","20"],["5","6"],["13","14"],["7","8"],["14","15"],["25","26"],["26","27"],["27","28"],["28","29"],["29","30"],["6","7"],["10","11"],["9","10"],["1","2"],["3","4"],["12","13"],["2","3"],["11","12"],["4","5"]],"fullPath":true}'
./hey -n $NR -c $NW -d "${DATA}" -m POST -T 'application/json' http://localhost:8080/flight
```

Using 10 workers in the server, and making 100k requests with `hey` using 100 workers, it answered with:
```
Summary:
  Total:	4.0668 secs
  Slowest:	0.1344 secs
  Fastest:	0.0001 secs
  Average:	0.0040 secs
  Requests/sec:	24589.2420
  
  Total data:	4547628 bytes
  Size/request:	45 bytes

Response time histogram:
  0.000 [1]	|
  0.014 [97384]	|■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.027 [44]	|
  0.040 [0]	|
  0.054 [0]	|
  0.067 [0]	|
  0.081 [0]	|
  0.094 [0]	|
  0.108 [1397]	|■
  0.121 [780]	|
  0.134 [394]	|


Latency distribution:
  10% in 0.0005 secs
  25% in 0.0007 secs
  50% in 0.0010 secs
  75% in 0.0015 secs
  90% in 0.0023 secs
  95% in 0.0033 secs
  99% in 0.1084 secs

Details (average, fastest, slowest):
  DNS+dialup:	0.0000 secs, 0.0001 secs, 0.1344 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0013 secs
  req write:	0.0000 secs, 0.0000 secs, 0.0030 secs
  resp wait:	0.0039 secs, 0.0001 secs, 0.1344 secs
  resp read:	0.0001 secs, 0.0000 secs, 0.0030 secs

Status code distribution:
  [200]	100000 responses
```

## Considerations

* An async log library could be used to detach the request time from writing the logs to the specified output.
* One could use a smaller and faster protocol such as gRPC or BSON instead of a REST full API.
* A faster and more high performant HTTP library could be used instead of `actix`, like hyper (https://hyper.rs/)
* The server number of threads is only controlled by the number of workers that can be selected by the env var `WORKERS`, a better parallelization could be done with better production tests.
Also the number of cores in the node running it could be considered.
* The service does not persist anything or read from any other source, this request is basically CPU bound.
  In case one needs to handle scenarios where there are many requests and processing them takes too long, one could change the interface to answer with a receipt id.
  Each request would save the flight info in a persistence and process it async.
  With the receipt id the client would be able to make another request to get the response once it is done, for example:
```shell
$ curl -X POST -H 'Content-Type: application/json' localhost:8080/flight -d '{"legs":[["IND","EWR"],["SFO", "ATL"],["GSO", "IND"],["ATL", "GSO"]],"full_path":true}'
{"type":"Ok","receipt_id":"506909ea-301e-4423-8710-e8559429f768"}
$ curl -X GET -H 'Content-Type: application/json' localhost:8080/flight -d '{"receipt_id":"506909ea-301e-4423-8710-e8559429f768"}'
{"type":"Ok","status":"NotProcessedYet"}
$ curl -X GET -H 'Content-Type: application/json' localhost:8080/flight -d '{"receipt_id":"506909ea-301e-4423-8710-e8559429f768"}'
{"type":"Ok","source":"SFO","destination":"EWR","path":["SFO","ATL","GSO","IND","EWR"]}
```
* The service runs stateless which makes easy to add a load balancer before it and horizontally scale the solution.
* The operations on the endpoints `/metrics` and `/health` can be split in another HTTP server so they do not compete with the load on the `/flight` endpoint.
* A client could ask for the path still left given the current airport of the passenger.
* A weight could be added to each leg of the flight and giving the current airport the passenger is, the service could answer with how long the flight will still take.
