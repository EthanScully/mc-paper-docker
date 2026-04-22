if [ "$2" != "linux/amd64" ]; then
    echo "Unsupported platform: $2"
    exit 1
fi
if [ "$1" = "linux/arm64" ]; then
    dpkg --add-architecture arm64 &&
        apt update &&
        apt install -y gcc gcc-aarch64-linux-gnu pkg-config &&
        rustup target add aarch64-unknown-linux-gnu &&
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
            cargo build -r --target aarch64-unknown-linux-gnu
elif [ "$1" = "linux/amd64" ]; then
    apt update &&
        apt install -y gcc pkg-config &&
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=gcc \
            cargo build -r --target x86_64-unknown-linux-gnu
else
    echo "Unsupported platform: $1"
    exit 1
fi &&
    mkdir -p /target/root/bin &&
    if [ "$1" = "linux/arm64" ]; then
        mv /build/target/aarch64-unknown-linux-gnu/release/mc-update /target/root/bin/mc-update
    elif [ "$1" = "linux/amd64" ]; then
        mv /build/target/x86_64-unknown-linux-gnu/release/mc-update /target/root/bin/mc-update
    fi