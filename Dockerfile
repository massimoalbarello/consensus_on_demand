### install packages
FROM rust:bullseye AS deps
WORKDIR /replica

RUN cargo install cargo-chef

### prepare the build
FROM deps AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

### build the ficc
FROM deps AS builder 
COPY --from=planner /replica/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build

### run the ficc replica
FROM deps AS runner
WORKDIR /replica
# copy the ficc binary
COPY --from=builder /replica/target/debug/consensus_on_demand .

EXPOSE 56789 56790

ENTRYPOINT ["./consensus_on_demand"]
