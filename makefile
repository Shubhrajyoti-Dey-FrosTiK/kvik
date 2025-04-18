run-1:
	cargo run -- --host-port 8080 -o 8081 -o 8082 -o 8083

run-2:
	cargo run -- --host-port 8081 -o 8080 -o 8082 -o 8083

run-3:
	cargo run -- --host-port 8082 -o 8081 -o 8080 -o 8083

run-4:
	cargo run -- --host-port 8083 -o 8081 -o 8082 -o 8080