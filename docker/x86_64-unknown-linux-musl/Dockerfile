FROM rustembedded/cross:x86_64-unknown-linux-musl-0.2.1

RUN apt update --fix-missing -y && \
    apt install -y openssl libssl-dev

RUN ln -s /usr/lib/x86_64-linux-gnu/libssl.so /usr/lib/x86_64-linux-gnu/libssl.so.1.0.0 && \
    ln -s /usr/lib/x86_64-linux-gnu/libcrypto.so /usr/lib/x86_64-linux-gnu/libcrypto.so.1.0.0