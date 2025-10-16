FROM rust:slim-bullseye

COPY . .
RUN cargo build --release

CMD ["./target/release/chat-plus-plus"]

EXPOSE 8080
