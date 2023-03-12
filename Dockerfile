FROM rust:bullseye

COPY ./ ./
RUN cargo build --release

EXPOSE 56789

CMD ["./target/release/consensus_on_demand", "--r", "1", "--n", "2", "--f", "0", "--p", "0", "--t", "100", "--d", "1000", "--cod"]