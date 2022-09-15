FROM rust:1.63-bullseye

WORKDIR /usr/src/acore-graphql

COPY . .

RUN cargo install --path . && cargo clean

CMD ["acore-graphql"]