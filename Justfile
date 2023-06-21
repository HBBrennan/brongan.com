_default:
  just --list

build:
	nix build .#dockerImage
	./result | docker load

run:
  nix run

deploy:
  just build
  podman push server docker://registry.fly.io/still-lake-5553:latest
  flyctl deploy -i registry.fly.io/still-lake-5553:latest
