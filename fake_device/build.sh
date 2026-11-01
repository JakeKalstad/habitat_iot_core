cargo build --release --target x86_64-unknown-linux-musl
docker build -t habitat-fake-device:local .
echo "1"