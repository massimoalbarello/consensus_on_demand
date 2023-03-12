FROM rust:bullseye

COPY ./ ./
RUN cargo build --release

EXPOSE 56789

ENTRYPOINT ["./target/release/consensus_on_demand"]