start redis docker:
docker run --name redis_1 --rm -p 6379:6379 -it redis:6 -- --loglevel verbose

cargo watch:
cargo watch -q -c -x run