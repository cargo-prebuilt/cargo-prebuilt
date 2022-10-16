default:
    just -l

pwd := `pwd`

docker:
    docker run -it --rm --pull=always \
    -v {{pwd}}:/prebuilt \
    rust:latest \
    bash
