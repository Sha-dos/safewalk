cross build --release --target aarch64-unknown-linux-gnu
#scp ./target/aarch64-unknown-linux-gnu/release/safewalk pi@192.168.68.109:~/
scp ./target/aarch64-unknown-linux-gnu/release/safewalk pi@192.168.2.3:~/
#cd frontend && npm run build
#scp -r frontend/out/* pi@192.168.68.109:~/frontend
