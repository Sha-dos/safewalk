cross build --release --target aarch64-unknown-linux-gnu
scp ./target/aarch64-unknown-linux-gnu/release/safewalk pi@192.168.68.101:~/
