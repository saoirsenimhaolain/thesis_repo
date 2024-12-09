FROM rust:1.80.1

COPY . .

EXPOSE 8080:8080

RUN cargo build --release

CMD [ "./target/release/rust-fannkuch" ]