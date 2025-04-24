run-1:
	cargo run -- --host-port 8080 -o 8081 -o 8082 -o 8083 -o 8084

run-2:
	cargo run -- --host-port 8081 -o 8080 -o 8082 -o 8083 -o 8084

run-3:
	cargo run -- --host-port 8082 -o 8081 -o 8080 -o 8083 -o 8084

run-4:
	cargo run -- --host-port 8083 -o 8081 -o 8082 -o 8080 -o 8084

run-5:
	cargo run -- --host-port 8084 -o 8081 -o 8082 -o 8080 -o 8083

run-release-1:
	./target/release/kv --host-port 8080 -o 8081 -o 8082 -o 8083 -o 8084

run-release-2:
	./target/release/kv --host-port 8081 -o 8080 -o 8082 -o 8083 -o 8084

run-release-3:
	./target/release/kv --host-port 8082 -o 8081 -o 8080 -o 8083 -o 8084

run-release-4:
	./target/release/kv --host-port 8083 -o 8081 -o 8082 -o 8080 -o 8084

run-release-5:
	./target/release/kv --host-port 8084 -o 8081 -o 8082 -o 8080 -o 8083

run-parallel:
	seq 1 5 | parallel -j 5 'make run-{} > ./logs/run_{}.log 2>&1'

run-release-parallel:
	seq 1 5 | parallel -j 5 'make run-release-{} > ./logs/run_{}.log 2>&1'
