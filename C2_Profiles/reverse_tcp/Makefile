build:
	cd reverse_tcp/mythic && cargo build --release --target x86_64-unknown-linux-musl
	cd reverse_tcp/c2_code && cargo build --release --target x86_64-unknown-linux-musl
	cp reverse_tcp/c2_code/target/x86_64-unknown-linux-musl/release/reverse_tcp_server reverse_tcp/bin
	cp reverse_tcp/mythic/target/x86_64-unknown-linux-musl/release/reverse_tcp_profile reverse_tcp/bin


clean:
	cd reverse_tcp/mythic && cargo clean
	cd reverse_tcp/c2_code && cargo clean
	- rm -f reverse_tcp/bin/reverse_tcp_server
	- rm -f reverse_tcp/bin/reverse_tcp_profile
	- rm -rf reverse_tcp/bin/logs


run:
	cp /reverse_tcp_profile reverse_tcp/bin/reverse_tcp_profile
	cp /reverse_tcp_server reverse_tcp/bin/reverse_tcp_server
	cd reverse_tcp/bin/ && ./reverse_tcp_profile


stop:
	pkill -f reverse_tcp_profile
	pkill -f reverse_tcp_server
