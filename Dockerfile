FROM rust:bullseye

WORKDIR /replica

COPY ./ ./
RUN cargo build

EXPOSE 56789

ENTRYPOINT ["./target/debug/consensus_on_demand"]